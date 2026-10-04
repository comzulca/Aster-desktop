//! Closed byte-snapshot processing. Complex decoders run only in the fixed
//! resource-limited child; this module grants no file, network or model authority.

use image::{AnimationDecoder, DynamicImage, ImageDecoder, ImageEncoder, ImageFormat};
use sha2::{Digest, Sha256};
use std::io::{Cursor, Write};
use std::sync::atomic::AtomicBool;

pub const MAX_SOURCE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_BATCH_SOURCE_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_BATCH_FILES: usize = 4;
pub const MAX_TEXT_BYTES: usize = 128 * 1024;
pub const MAX_BATCH_TEXT_BYTES: usize = 256 * 1024;
pub const MAX_DIMENSION: u32 = 4096;
pub const MAX_PIXELS: u64 = 4_194_304;
pub const MAX_PDF_PAGES: usize = 20;
pub const PDF_DPI: u32 = 150;
pub const MAX_PNG_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_RGBA_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_WORKER_OUTPUT_BYTES: usize = MAX_RGBA_BYTES + MAX_PNG_BYTES + 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum InputKind {
    Text = 1,
    Png = 2,
    Jpeg = 3,
    Gif = 4,
    Pdf = 5,
}
impl TryFrom<u8> for InputKind {
    type Error = ProcessingError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Text),
            2 => Ok(Self::Png),
            3 => Ok(Self::Jpeg),
            4 => Ok(Self::Gif),
            5 => Ok(Self::Pdf),
            _ => Err(ProcessingError::Invalid),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcessingError {
    Invalid,
    Limit,
    Unsupported,
    Isolation,
    Cancelled,
    Timeout,
    Worker,
}

#[derive(Debug)]
pub struct ProcessedPage {
    pub width: u32,
    pub height: u32,
    /// RGBA from the same pixels used for the canonical metadata-free PNG.
    pub rgba: Vec<u8>,
    pub png: Vec<u8>,
}
#[derive(Debug)]
pub enum ProcessedContent {
    Text(String),
    Pages(Vec<ProcessedPage>),
}
#[derive(Debug)]
pub struct ProcessedAttachment {
    /// Witness of the parent's immutable original. It is not original storage.
    pub source_sha256: String,
    pub content: ProcessedContent,
}

pub fn source_hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

pub fn validate_dimensions(width: u32, height: u32) -> Result<usize, ProcessingError> {
    let pixels = u64::from(width) * u64::from(height);
    if width == 0
        || height == 0
        || width > MAX_DIMENSION
        || height > MAX_DIMENSION
        || pixels > MAX_PIXELS
    {
        return Err(ProcessingError::Limit);
    }
    usize::try_from(pixels * 4).map_err(|_| ProcessingError::Limit)
}

pub fn validate_source(kind: InputKind, bytes: &[u8]) -> Result<(), ProcessingError> {
    if bytes.is_empty() || bytes.len() > MAX_SOURCE_BYTES {
        return Err(ProcessingError::Limit);
    }
    let signature = match kind {
        InputKind::Text => {
            return validate_text(bytes).map(|_| ());
        }
        InputKind::Png => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        InputKind::Jpeg => bytes.starts_with(b"\xff\xd8\xff"),
        InputKind::Gif => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        InputKind::Pdf => bytes.starts_with(b"%PDF-"),
    };
    if signature {
        Ok(())
    } else {
        Err(ProcessingError::Invalid)
    }
}

fn validate_text(bytes: &[u8]) -> Result<&str, ProcessingError> {
    if bytes.is_empty() || bytes.len() > MAX_TEXT_BYTES {
        return Err(ProcessingError::Limit);
    }
    // A renamed PDF/Office ZIP/image is not inert-text format admission. In
    // particular an ASCII PDF must not bypass full-page native/visual checks.
    if bytes.starts_with(b"%PDF-")
        || bytes.starts_with(b"PK\x03\x04")
        || bytes.starts_with(b"PK\x05\x06")
        || bytes.starts_with(b"PK\x07\x08")
        || bytes.starts_with(b"\x89PNG\r\n\x1a\n")
        || bytes.starts_with(b"\xff\xd8\xff")
        || bytes.starts_with(b"GIF87a")
        || bytes.starts_with(b"GIF89a")
    {
        return Err(ProcessingError::Invalid);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| ProcessingError::Invalid)?;
    if text.contains('\0') {
        return Err(ProcessingError::Invalid);
    }
    Ok(text)
}

/// Only the authenticated host may call this after identity-pinned selection.
/// The fixed worker is not a network sandbox; see ADR-0019's residual gap.
pub fn process_isolated(
    kind: InputKind,
    original: &[u8],
    cancel: &AtomicBool,
) -> Result<ProcessedAttachment, ProcessingError> {
    validate_source(kind, original)?;
    let output = crate::attachment_worker::run_isolated(kind, original, cancel)?;
    crate::attachment_worker::decode_response(kind, original, &output)
}

struct LimitedPng {
    bytes: Vec<u8>,
}
impl Write for LimitedPng {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        if self
            .bytes
            .len()
            .checked_add(bytes.len())
            .is_none_or(|n| n > MAX_PNG_BYTES)
        {
            return Err(std::io::Error::other("Normalised image limit"));
        }
        self.bytes.extend_from_slice(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Safe bounded encoder, not an untrusted encoded-input decoder. The parent
/// regenerates and compares this PNG so raw preview pixels and remote PNG agree.
pub(crate) fn page_from_rgba(
    width: u32,
    height: u32,
    rgba: Vec<u8>,
) -> Result<ProcessedPage, ProcessingError> {
    if rgba.len() != validate_dimensions(width, height)? {
        return Err(ProcessingError::Invalid);
    }
    let mut png = LimitedPng { bytes: Vec::new() };
    image::codecs::png::PngEncoder::new(&mut png)
        .write_image(&rgba, width, height, image::ExtendedColorType::Rgba8)
        .map_err(|_| ProcessingError::Limit)?;
    Ok(ProcessedPage {
        width,
        height,
        rgba,
        png: png.bytes,
    })
}

pub(crate) fn validate_pages(pages: &[ProcessedPage]) -> Result<(), ProcessingError> {
    if pages.is_empty() || pages.len() > MAX_PDF_PAGES {
        return Err(ProcessingError::Limit);
    }
    let mut rgba_total = 0usize;
    let mut png_total = 0usize;
    for page in pages {
        if page.rgba.len() != validate_dimensions(page.width, page.height)? {
            return Err(ProcessingError::Invalid);
        }
        rgba_total = rgba_total
            .checked_add(page.rgba.len())
            .ok_or(ProcessingError::Limit)?;
        png_total = png_total
            .checked_add(page.png.len())
            .ok_or(ProcessingError::Limit)?;
        if page.png.is_empty() || rgba_total > MAX_RGBA_BYTES || png_total > MAX_PNG_BYTES {
            return Err(ProcessingError::Limit);
        }
    }
    Ok(())
}

fn limits() -> image::Limits {
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    // Decoder allocation limits are best-effort. The child Job memory limit
    // independently hard-bounds total native/decoder allocations.
    limits.max_alloc = Some(MAX_RGBA_BYTES as u64);
    limits
}

fn decode_static_image(kind: InputKind, bytes: &[u8]) -> Result<ProcessedPage, ProcessingError> {
    validate_source(kind, bytes)?;
    let rgba = match kind {
        InputKind::Gif => {
            let mut decoder = image::codecs::gif::GifDecoder::new(Cursor::new(bytes))
                .map_err(|_| ProcessingError::Invalid)?;
            decoder
                .set_limits(limits())
                .map_err(|_| ProcessingError::Limit)?;
            validate_dimensions(decoder.dimensions().0, decoder.dimensions().1)?;
            let mut frames = decoder.into_frames();
            let first = frames
                .next()
                .ok_or(ProcessingError::Invalid)?
                .map_err(|_| ProcessingError::Invalid)?;
            // Any second frame or later decoding error rejects the entire GIF.
            if frames.next().is_some() {
                return Err(ProcessingError::Unsupported);
            }
            first.into_buffer()
        }
        InputKind::Png | InputKind::Jpeg => {
            let format = if kind == InputKind::Png {
                ImageFormat::Png
            } else {
                ImageFormat::Jpeg
            };
            if kind == InputKind::Png {
                let decoder =
                    image::codecs::png::PngDecoder::with_limits(Cursor::new(bytes), limits())
                        .map_err(|_| ProcessingError::Invalid)?;
                if decoder.is_apng().map_err(|_| ProcessingError::Invalid)? {
                    return Err(ProcessingError::Unsupported);
                }
            }
            let mut reader = image::ImageReader::with_format(Cursor::new(bytes), format);
            reader.limits(limits());
            let mut decoder = reader
                .into_decoder()
                .map_err(|_| ProcessingError::Invalid)?;
            let (width, height) = decoder.dimensions();
            validate_dimensions(width, height)?;
            let orientation = decoder
                .orientation()
                .map_err(|_| ProcessingError::Invalid)?;
            let mut decoded =
                DynamicImage::from_decoder(decoder).map_err(|_| ProcessingError::Invalid)?;
            decoded.apply_orientation(orientation);
            decoded.into_rgba8()
        }
        _ => return Err(ProcessingError::Invalid),
    };
    page_from_rgba(rgba.width(), rgba.height(), rgba.into_raw())
}

/// Parser entry ONLY for the verified child mode (and synthetic unit fixtures).
pub(crate) fn process_in_worker(
    kind: InputKind,
    bytes: &[u8],
) -> Result<ProcessedAttachment, ProcessingError> {
    validate_source(kind, bytes)?;
    let content = match kind {
        InputKind::Text => ProcessedContent::Text(validate_text(bytes)?.to_owned()),
        InputKind::Png | InputKind::Jpeg | InputKind::Gif => {
            let pages = vec![decode_static_image(kind, bytes)?];
            validate_pages(&pages)?;
            ProcessedContent::Pages(pages)
        }
        InputKind::Pdf => ProcessedContent::Pages(render_pdf(bytes)?),
    };
    Ok(ProcessedAttachment {
        source_sha256: source_hash(bytes),
        content,
    })
}

#[cfg(not(windows))]
fn render_pdf(_: &[u8]) -> Result<Vec<ProcessedPage>, ProcessingError> {
    Err(ProcessingError::Unsupported)
}

#[cfg(windows)]
fn render_pdf(bytes: &[u8]) -> Result<Vec<ProcessedPage>, ProcessingError> {
    use windows::Data::Pdf::{PdfDocument, PdfPageRenderOptions};
    use windows::Graphics::Imaging::BitmapEncoder;
    use windows::Storage::Streams::{DataReader, DataWriter, InMemoryRandomAccessStream};
    #[link(name = "runtimeobject")]
    unsafe extern "system" {
        fn RoInitialize(init_type: u32) -> i32;
        fn RoUninitialize();
    }
    struct Apartment;
    impl Drop for Apartment {
        fn drop(&mut self) {
            unsafe {
                RoUninitialize();
            }
        }
    }
    // Dedicated child thread, before any Tauri/native app bootstrap. No STA pump.
    if unsafe { RoInitialize(1) } < 0 {
        return Err(ProcessingError::Worker);
    }
    let _apartment = Apartment;
    let fail = |_| ProcessingError::Invalid;
    let stream = InMemoryRandomAccessStream::new().map_err(fail)?;
    let writer = DataWriter::CreateDataWriter(&stream).map_err(fail)?;
    writer.WriteBytes(bytes).map_err(fail)?;
    if writer.StoreAsync().map_err(fail)?.join().map_err(fail)? as usize != bytes.len() {
        return Err(ProcessingError::Invalid);
    }
    writer.DetachStream().map_err(fail)?;
    stream.Seek(0).map_err(fail)?;
    let document = PdfDocument::LoadFromStreamAsync(&stream)
        .map_err(fail)?
        .join()
        .map_err(fail)?;
    if document.IsPasswordProtected().map_err(fail)? {
        return Err(ProcessingError::Unsupported);
    }
    let count = document.PageCount().map_err(fail)? as usize;
    if count == 0 || count > MAX_PDF_PAGES {
        return Err(ProcessingError::Limit);
    }
    let mut pages = Vec::with_capacity(count);
    for index in 0..count {
        let page = document.GetPage(index as u32).map_err(fail)?;
        let size = page.Size().map_err(fail)?;
        // Windows PDF sizes are device-independent pixels (96 DPI).
        let width_f = (f64::from(size.Width) * f64::from(PDF_DPI) / 96.0).ceil();
        let height_f = (f64::from(size.Height) * f64::from(PDF_DPI) / 96.0).ceil();
        if !width_f.is_finite()
            || !height_f.is_finite()
            || width_f < 1.0
            || height_f < 1.0
            || width_f > f64::from(MAX_DIMENSION)
            || height_f > f64::from(MAX_DIMENSION)
        {
            return Err(ProcessingError::Limit);
        }
        let (width, height) = (width_f as u32, height_f as u32);
        let next_rgba = validate_dimensions(width, height)?;
        let previous_rgba: usize = pages.iter().map(|p: &ProcessedPage| p.rgba.len()).sum();
        if previous_rgba
            .checked_add(next_rgba)
            .is_none_or(|n| n > MAX_RGBA_BYTES)
        {
            return Err(ProcessingError::Limit);
        }
        let options = PdfPageRenderOptions::new().map_err(fail)?;
        options
            .SetBitmapEncoderId(BitmapEncoder::PngEncoderId().map_err(fail)?)
            .map_err(fail)?;
        options.SetIsIgnoringHighContrast(true).map_err(fail)?;
        options.SetDestinationWidth(width).map_err(fail)?;
        options.SetDestinationHeight(height).map_err(fail)?;
        let output = InMemoryRandomAccessStream::new().map_err(fail)?;
        page.RenderWithOptionsToStreamAsync(&output, &options)
            .map_err(fail)?
            .join()
            .map_err(fail)?;
        let length =
            usize::try_from(output.Size().map_err(fail)?).map_err(|_| ProcessingError::Limit)?;
        if length == 0 || length > MAX_SOURCE_BYTES {
            return Err(ProcessingError::Limit);
        }
        let input = output.GetInputStreamAt(0).map_err(fail)?;
        let reader = DataReader::CreateDataReader(&input).map_err(fail)?;
        if reader
            .LoadAsync(length as u32)
            .map_err(fail)?
            .join()
            .map_err(fail)? as usize
            != length
        {
            return Err(ProcessingError::Invalid);
        }
        let mut rendered = vec![0; length];
        reader.ReadBytes(&mut rendered).map_err(fail)?;
        let normalised = decode_static_image(InputKind::Png, &rendered)?;
        if normalised.width != width || normalised.height != height {
            return Err(ProcessingError::Invalid);
        }
        pages.push(normalised);
        validate_pages(&pages)?;
        page.Close().map_err(fail)?;
    }
    if pages.len() != count {
        return Err(ProcessingError::Invalid);
    }
    Ok(pages)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dimensions_and_signature_fail_closed() {
        assert_eq!(validate_dimensions(2048, 2048).unwrap(), 16_777_216);
        for (w, h) in [
            (0, 1),
            (1, 0),
            (4097, 1),
            (2049, 2048),
            (u32::MAX, u32::MAX),
        ] {
            assert_eq!(validate_dimensions(w, h), Err(ProcessingError::Limit));
        }
        assert_eq!(
            validate_source(InputKind::Png, b"GIF89a"),
            Err(ProcessingError::Invalid)
        );
        assert_eq!(InputKind::try_from(6), Err(ProcessingError::Invalid));
    }
    #[test]
    fn text_stays_inert_and_bounded() {
        let text = b"<!DOCTYPE x [<!ENTITY y SYSTEM 'file:///secret'>]>\n=RUN()";
        let processed = process_in_worker(InputKind::Text, text).unwrap();
        assert!(matches!(processed.content, ProcessedContent::Text(ref s) if s.as_bytes() == text));
        assert_eq!(
            validate_source(InputKind::Text, &[0xff]),
            Err(ProcessingError::Invalid)
        );
        assert_eq!(
            validate_source(InputKind::Text, b"a\0b"),
            Err(ProcessingError::Invalid)
        );
        assert_eq!(
            validate_source(InputKind::Text, &vec![b'x'; MAX_TEXT_BYTES + 1]),
            Err(ProcessingError::Limit)
        );
        for signature in [
            b"%PDF-1.4\nASCII PDF".as_slice(),
            b"PK\x03\x04".as_slice(),
            b"GIF89a".as_slice(),
        ] {
            assert_eq!(
                validate_source(InputKind::Text, signature),
                Err(ProcessingError::Invalid)
            );
        }
    }
    #[test]
    fn png_roundtrip_is_metadata_free_and_preserves_pixels() {
        let rgba = vec![255, 0, 0, 255, 0, 0, 255, 255];
        let page = page_from_rgba(2, 1, rgba.clone()).unwrap();
        let processed = process_in_worker(InputKind::Png, &page.png).unwrap();
        let ProcessedContent::Pages(pages) = processed.content else {
            panic!("pages")
        };
        assert_eq!(pages.len(), 1);
        assert_eq!(pages[0].rgba, rgba);
        assert_eq!(pages[0].png, page.png);
        assert!(!pages[0]
            .png
            .windows(4)
            .any(|w| [b"eXIf", b"tEXt", b"iCCP"].iter().any(|tag| w == *tag)));
    }

    fn png_chunk(kind: &[u8; 4], body: &[u8]) -> Vec<u8> {
        let mut chunk = (body.len() as u32).to_be_bytes().to_vec();
        chunk.extend_from_slice(kind);
        chunk.extend_from_slice(body);
        let mut crc = !0u32;
        for byte in &chunk[4..] {
            crc ^= u32::from(*byte);
            for _ in 0..8 {
                crc = (crc >> 1) ^ (0xedb8_8320 & 0u32.wrapping_sub(crc & 1));
            }
        }
        chunk.extend_from_slice(&(!crc).to_be_bytes());
        chunk
    }

    #[test]
    fn source_png_text_metadata_is_not_copied_and_apng_is_refused() {
        let canonical = page_from_rgba(1, 1, vec![20, 30, 40, 255]).unwrap();
        // Insert a valid inert ancillary text chunk after the complete IHDR.
        let mut source = canonical.png.clone();
        source.splice(33..33, png_chunk(b"tEXt", b"Label\0synthetic-metadata"));
        let ProcessedContent::Pages(pages) =
            process_in_worker(InputKind::Png, &source).unwrap().content
        else {
            panic!("pages")
        };
        assert_eq!(pages[0].png, canonical.png);
        assert!(!pages[0].png.windows(18).any(|w| w == b"synthetic-metadata"));
        let mut animation_control = Vec::new();
        animation_control.extend(1u32.to_be_bytes());
        animation_control.extend(0u32.to_be_bytes());
        let mut frame_control = Vec::new();
        for n in [0u32, 1, 1, 0, 0] {
            frame_control.extend(n.to_be_bytes());
        }
        frame_control.extend(1u16.to_be_bytes());
        frame_control.extend(100u16.to_be_bytes());
        frame_control.extend([0, 0]);
        let mut chunks = png_chunk(b"acTL", &animation_control);
        chunks.extend(png_chunk(b"fcTL", &frame_control));
        let mut apng = canonical.png;
        apng.splice(33..33, chunks);
        assert!(matches!(
            process_in_worker(InputKind::Png, &apng),
            Err(ProcessingError::Unsupported)
        ));
    }
    #[test]
    fn multi_frame_gif_is_refused_not_truncated() {
        let mut bytes = Vec::new();
        {
            let mut encoder = image::codecs::gif::GifEncoder::new(&mut bytes);
            for _ in 0..2 {
                encoder
                    .encode_frame(image::Frame::new(image::RgbaImage::from_pixel(
                        1,
                        1,
                        image::Rgba([1, 2, 3, 255]),
                    )))
                    .unwrap();
            }
        }
        assert!(matches!(
            process_in_worker(InputKind::Gif, &bytes),
            Err(ProcessingError::Unsupported)
        ));
    }

    #[test]
    fn static_gif_and_jpeg_become_full_size_png() {
        let mut gif = Vec::new();
        image::codecs::gif::GifEncoder::new(&mut gif)
            .encode_frame(image::Frame::new(image::RgbaImage::from_pixel(
                3,
                2,
                image::Rgba([20, 80, 130, 255]),
            )))
            .unwrap();
        let mut jpeg = Vec::new();
        image::codecs::jpeg::JpegEncoder::new(&mut jpeg)
            .encode(&[50u8; 18], 3, 2, image::ExtendedColorType::Rgb8)
            .unwrap();
        for (kind, bytes) in [(InputKind::Gif, gif), (InputKind::Jpeg, jpeg)] {
            let ProcessedContent::Pages(pages) = process_in_worker(kind, &bytes).unwrap().content
            else {
                panic!("pages")
            };
            assert_eq!(pages.len(), 1);
            assert_eq!((pages[0].width, pages[0].height), (3, 2));
            assert_eq!(pages[0].rgba.len(), 24);
            assert!(pages[0].png.starts_with(b"\x89PNG\r\n\x1a\n"));
        }
    }

    #[test]
    fn rgba_and_page_aggregate_limits_are_fail_closed() {
        assert!(matches!(
            page_from_rgba(1, 1, vec![0; 3]),
            Err(ProcessingError::Invalid)
        ));
        assert!(matches!(validate_pages(&[]), Err(ProcessingError::Limit)));
        let pages: Vec<_> = (0..=MAX_PDF_PAGES)
            .map(|_| page_from_rgba(1, 1, vec![0, 0, 0, 255]).unwrap())
            .collect();
        assert!(matches!(
            validate_pages(&pages),
            Err(ProcessingError::Limit)
        ));
        let oversized = ProcessedPage {
            width: 1,
            height: 1,
            rgba: vec![0, 0, 0, 255],
            png: vec![1; MAX_PNG_BYTES + 1],
        };
        assert!(matches!(
            validate_pages(&[oversized]),
            Err(ProcessingError::Limit)
        ));
    }

    #[cfg(windows)]
    fn synthetic_pdf(page_count: usize) -> Vec<u8> {
        let mut objects = vec![
            "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
            format!(
                "<< /Type /Pages /Count {page_count} /Kids [{}] >>",
                (0..page_count)
                    .map(|index| format!("{} 0 R", 3 + index * 2))
                    .collect::<Vec<_>>()
                    .join(" ")
            ),
        ];
        for index in 0..page_count {
            let content_id = 4 + index * 2;
            objects.push(format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 72 72] /Resources << >> /Contents {content_id} 0 R >>"));
            let paint = if index % 2 == 0 {
                "1 0 0 rg\n0 0 72 72 re f\n"
            } else {
                "0 0 1 rg\n0 0 72 72 re f\n"
            };
            objects.push(format!(
                "<< /Length {} >>\nstream\n{paint}endstream",
                paint.len()
            ));
        }
        let mut pdf = b"%PDF-1.4\n".to_vec();
        let mut offsets = vec![0usize];
        for (index, object) in objects.iter().enumerate() {
            offsets.push(pdf.len());
            pdf.extend(format!("{} 0 obj\n{object}\nendobj\n", index + 1).as_bytes());
        }
        let xref = pdf.len();
        pdf.extend(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len()).as_bytes());
        for offset in &offsets[1..] {
            pdf.extend(format!("{offset:010} 00000 n \n").as_bytes());
        }
        pdf.extend(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
                offsets.len()
            )
            .as_bytes(),
        );
        pdf
    }

    #[cfg(windows)]
    #[test]
    fn synthetic_pdf_retains_all_pages_in_order_and_rejects_over_limit() {
        let original = synthetic_pdf(2);
        let processed = process_in_worker(InputKind::Pdf, &original).unwrap();
        assert_eq!(processed.source_sha256, source_hash(&original));
        let ProcessedContent::Pages(pages) = processed.content else {
            panic!("pages")
        };
        assert_eq!(pages.len(), 2);
        assert_eq!((pages[0].width, pages[0].height), (150, 150));
        let centre = (75 * 150 + 75) * 4;
        assert_eq!(&pages[0].rgba[centre..centre + 4], &[255, 0, 0, 255]);
        assert_eq!(&pages[1].rgba[centre..centre + 4], &[0, 0, 255, 255]);
        assert!(matches!(
            process_in_worker(InputKind::Pdf, &synthetic_pdf(MAX_PDF_PAGES + 1)),
            Err(ProcessingError::Limit)
        ));
    }
}
