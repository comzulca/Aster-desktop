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

/// Failure location only, never a success receipt or native error message.
/// Reserved nonzero child codes disclose no source/path/environment/HRESULT.
/// Covers mapped native stream/WinRT/normalisation errors; invariant/limit guards stay generic.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum PdfFailureStage {
    InitialiseRuntime = 64,
    CreateInputStream = 65,
    CreateInputWriter = 66,
    WriteInput = 67,
    StoreInput = 68,
    DetachInput = 69,
    SeekInput = 70,
    LoadDocument = 71,
    InspectDocument = 72,
    ReadPage = 73,
    ReadPageSize = 74,
    CreateRenderOptions = 75,
    ConfigureRender = 76,
    CreateOutputStream = 77,
    RenderPage = 78,
    ReadOutput = 79,
    NormalisePage = 80,
    ClosePage = 81,
}
impl PdfFailureStage {
    pub const ALL: [Self; 18] = [
        Self::InitialiseRuntime,
        Self::CreateInputStream,
        Self::CreateInputWriter,
        Self::WriteInput,
        Self::StoreInput,
        Self::DetachInput,
        Self::SeekInput,
        Self::LoadDocument,
        Self::InspectDocument,
        Self::ReadPage,
        Self::ReadPageSize,
        Self::CreateRenderOptions,
        Self::ConfigureRender,
        Self::CreateOutputStream,
        Self::RenderPage,
        Self::ReadOutput,
        Self::NormalisePage,
        Self::ClosePage,
    ];
    pub fn worker_exit_code(self) -> u32 {
        self as u32
    }
    pub fn from_worker_exit(code: u32) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|stage| stage.worker_exit_code() == code)
    }
    pub fn diagnostic_reason(self) -> &'static str {
        match self {
            Self::InitialiseRuntime => "pdf-initialise-runtime",
            Self::CreateInputStream => "pdf-create-input-stream",
            Self::CreateInputWriter => "pdf-create-input-writer",
            Self::WriteInput => "pdf-write-input",
            Self::StoreInput => "pdf-store-input",
            Self::DetachInput => "pdf-detach-input",
            Self::SeekInput => "pdf-seek-input",
            Self::LoadDocument => "pdf-load-document",
            Self::InspectDocument => "pdf-inspect-document",
            Self::ReadPage => "pdf-read-page",
            Self::ReadPageSize => "pdf-read-page-size",
            Self::CreateRenderOptions => "pdf-create-render-options",
            Self::ConfigureRender => "pdf-configure-render",
            Self::CreateOutputStream => "pdf-create-output-stream",
            Self::RenderPage => "pdf-render-page",
            Self::ReadOutput => "pdf-read-output",
            Self::NormalisePage => "pdf-normalise-page",
            Self::ClosePage => "pdf-close-page",
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
    Pdf(PdfFailureStage),
}

#[cfg(any(windows, test))]
fn pdf_normalisation_error(error: ProcessingError) -> ProcessingError {
    match error {
        ProcessingError::Invalid | ProcessingError::Unsupported | ProcessingError::Worker => {
            ProcessingError::Pdf(PdfFailureStage::NormalisePage)
        }
        // Numeric bounds and boundary/cancellation categories remain intact.
        _ => error,
    }
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

/// Pixel decoding alone stops before the PNG container has necessarily reached
/// IEND. Validate the complete pinned-codec stream, including every chunk CRC,
/// compressed-data checksum and exact EOF, before normalisation. This is a
/// child-only complex parser; the parent continues to accept bounded RGBA only.
fn validate_complete_png(bytes: &[u8]) -> Result<(), ProcessingError> {
    let mut cursor = Cursor::new(bytes);
    {
        let mut options = png::DecodeOptions::default();
        options.set_ignore_checksums(false);
        options.set_skip_ancillary_crc_failures(false);
        options.set_ignore_text_chunk(true);
        options.set_ignore_iccp_chunk(true);
        let mut decoder = png::Decoder::new_with_options(&mut cursor, options);
        decoder.set_limits(png::Limits {
            bytes: MAX_RGBA_BYTES,
        });
        let header = decoder
            .read_header_info()
            .map_err(|_| ProcessingError::Invalid)?;
        validate_dimensions(header.width, header.height)?;
        let mut reader = decoder.read_info().map_err(|_| ProcessingError::Invalid)?;
        if reader.info().animation_control.is_some() || reader.info().frame_control.is_some() {
            return Err(ProcessingError::Unsupported);
        }
        let length = reader
            .output_buffer_size()
            .filter(|length| *length > 0 && *length <= MAX_RGBA_BYTES)
            .ok_or(ProcessingError::Limit)?;
        let mut pixels = vec![0; length];
        reader
            .next_frame(&mut pixels)
            .map_err(|_| ProcessingError::Invalid)?;
        reader.finish().map_err(|_| ProcessingError::Invalid)?;
        if reader.info().animation_control.is_some() || reader.info().frame_control.is_some() {
            return Err(ProcessingError::Unsupported);
        }
    }
    // The decoder uses Cursor::fill_buf + consume(actual bytes), not a read-
    // ahead wrapper. IEND followed by garbage or another PNG is not one input.
    if cursor.position() != bytes.len() as u64 {
        return Err(ProcessingError::Invalid);
    }
    Ok(())
}

/// `image` deliberately uses permissive JPEG recovery. Call the same pinned
/// JPEG codec directly in strict mode instead: damaged entropy must not become
/// grey/default-filled pixels. An independent EOI and consumed-byte check also
/// closes the codec's otherwise tolerated missing-end-marker path.
fn decode_complete_jpeg(bytes: &[u8]) -> Result<image::RgbaImage, ProcessingError> {
    use zune_jpeg::zune_core::{colorspace::ColorSpace, options::DecoderOptions};
    if !bytes.ends_with(b"\xff\xd9") {
        return Err(ProcessingError::Invalid);
    }
    let mut cursor = Cursor::new(bytes);
    let (width, height, pixels, orientation) = {
        let options = DecoderOptions::default()
            .set_strict_mode(true)
            .set_max_width(MAX_DIMENSION as usize)
            .set_max_height(MAX_DIMENSION as usize)
            .jpeg_set_out_colorspace(ColorSpace::RGB);
        let mut decoder = zune_jpeg::JpegDecoder::new_with_options(&mut cursor, options);
        decoder
            .decode_headers()
            .map_err(|_| ProcessingError::Invalid)?;
        let (width, height) = decoder.dimensions().ok_or(ProcessingError::Invalid)?;
        let width = u32::try_from(width).map_err(|_| ProcessingError::Limit)?;
        let height = u32::try_from(height).map_err(|_| ProcessingError::Limit)?;
        let expected = validate_dimensions(width, height)? / 4 * 3;
        if decoder.output_buffer_size() != Some(expected) {
            return Err(ProcessingError::Invalid);
        }
        let orientation = decoder
            .exif()
            .and_then(|exif| image::metadata::Orientation::from_exif_chunk(exif))
            .unwrap_or(image::metadata::Orientation::NoTransforms);
        let mut pixels = vec![0; expected];
        decoder
            .decode_into(&mut pixels)
            .map_err(|_| ProcessingError::Invalid)?;
        (width, height, pixels, orientation)
    };
    if cursor.position() != bytes.len() as u64 {
        return Err(ProcessingError::Invalid);
    }
    let rgb = image::RgbImage::from_raw(width, height, pixels).ok_or(ProcessingError::Invalid)?;
    let mut decoded = DynamicImage::ImageRgb8(rgb);
    decoded.apply_orientation(orientation);
    Ok(decoded.into_rgba8())
}

/// The image compositing adapter clips GIF frames to their logical screen and
/// stops at the first trailer. Prevalidate one complete, screen-consistent frame
/// with the pinned GIF codec so neither clipping nor suffix omission is allowed.
fn validate_complete_gif(bytes: &[u8]) -> Result<(), ProcessingError> {
    // EOF alone is not a GIF Trailer. Require the terminal marker in addition
    // to the decoder's exact consumed-byte and complete-LZW checks below.
    if !bytes.ends_with(b"\x3b") {
        return Err(ProcessingError::Invalid);
    }
    let mut options = gif::DecodeOptions::new();
    options.set_color_output(gif::ColorOutput::RGBA);
    options.set_memory_limit(gif::MemoryLimit::Bytes(
        std::num::NonZeroU64::new(MAX_RGBA_BYTES as u64).ok_or(ProcessingError::Limit)?,
    ));
    options.check_frame_consistency(true);
    options.check_lzw_end_code(true);
    let mut decoder = options
        .read_info(Cursor::new(bytes))
        .map_err(|_| ProcessingError::Invalid)?;
    validate_dimensions(u32::from(decoder.width()), u32::from(decoder.height()))?;
    let first = decoder
        .read_next_frame()
        .map_err(|_| ProcessingError::Invalid)?
        .ok_or(ProcessingError::Invalid)?;
    if first.buffer.len() != validate_dimensions(u32::from(first.width), u32::from(first.height))? {
        return Err(ProcessingError::Invalid);
    }
    if decoder
        .read_next_frame()
        .map_err(|_| ProcessingError::Invalid)?
        .is_some()
    {
        return Err(ProcessingError::Unsupported);
    }
    // gif::Decoder owns a BufReader which may have read ahead after Trailer.
    // Subtract its unread bytes; the underlying Cursor alone is not an EOF proof.
    let buffered = decoder.into_inner();
    let consumed = buffered
        .get_ref()
        .position()
        .checked_sub(buffered.buffer().len() as u64)
        .ok_or(ProcessingError::Invalid)?;
    if consumed != bytes.len() as u64 {
        return Err(ProcessingError::Invalid);
    }
    Ok(())
}

fn decode_static_image(kind: InputKind, bytes: &[u8]) -> Result<ProcessedPage, ProcessingError> {
    validate_source(kind, bytes)?;
    let rgba = match kind {
        InputKind::Gif => {
            validate_complete_gif(bytes)?;
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
        InputKind::Jpeg => decode_complete_jpeg(bytes)?,
        InputKind::Png => {
            validate_complete_png(bytes)?;
            let mut reader = image::ImageReader::with_format(Cursor::new(bytes), ImageFormat::Png);
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
fn admit_memory_stream_capabilities(can_read: bool, can_write: bool) -> windows::core::Result<()> {
    if can_read && can_write {
        Ok(())
    } else {
        Err(windows::core::Error::from_hresult(
            windows::Win32::Foundation::E_FAIL,
        ))
    }
}

#[cfg(windows)]
fn new_memory_backing_stream() -> windows::core::Result<windows::Win32::System::Com::IStream> {
    // Safety: None passes a null initial buffer and zero byte count, requesting
    // fresh private memory. COM reference ownership retains the allocation.
    // No caller-owned handle/buffer, file, custom loader or retry is involved.
    unsafe { windows::Win32::UI::Shell::SHCreateMemStream(None) }.ok_or_else(|| {
        windows::core::Error::from_hresult(windows::Win32::Foundation::E_OUTOFMEMORY)
    })
}

#[cfg(windows)]
fn new_memory_stream() -> windows::core::Result<windows::Storage::Streams::IRandomAccessStream> {
    use windows::Win32::System::WinRT::{CreateRandomAccessStreamOverStream, BSOS_DEFAULT};

    let backing = new_memory_backing_stream()?;
    // Safety: the fixed documented adapter receives our owned COM stream;
    // its typed interface retains the backing. The adapter option is unchanged.
    let stream: windows::Storage::Streams::IRandomAccessStream =
        unsafe { CreateRandomAccessStreamOverStream(&backing, BSOS_DEFAULT) }?;
    admit_memory_stream_capabilities(stream.CanRead()?, stream.CanWrite()?)?;
    Ok(stream)
}

// P10 SCOPED FACTORIES BEGIN
#[cfg(windows)]
mod scoped_pdf_factories {
    use windows::core::{factory, Interface, Param, Result, Type, GUID};
    use windows::Data::Pdf::{IPdfDocumentStatics, PdfDocument, PdfPageRenderOptions};
    use windows::Graphics::Imaging::{BitmapEncoder, IBitmapEncoderStatics};
    use windows::Storage::Streams::{
        DataReader, DataWriter, IDataReaderFactory, IDataWriterFactory, IInputStream,
        IOutputStream, IRandomAccessStream,
    };
    use windows::Win32::System::WinRT::IActivationFactory;

    // Fixed class/interface pairs and operation-local owned factories avoid the
    // generated static FactoryCache. The pinned upstream factory loader and its
    // inherited internal recovery behavior remain unchanged; no Aster retry or
    // caller-selected class, IID, path or cached COM lifetime is introduced.
    pub(super) fn data_writer<P0>(outputstream: P0) -> Result<DataWriter>
    where
        P0: Param<IOutputStream>,
    {
        let factory = factory::<DataWriter, IDataWriterFactory>()?;
        // Safety: the pinned generated vtable and Param ABI match exactly. The
        // borrowed parameter lives through the call; successful output transfers
        // ownership once with Type::from_abi, preserving failed/null semantics.
        unsafe {
            let mut result__ = core::mem::zeroed();
            (Interface::vtable(&factory).CreateDataWriter)(
                Interface::as_raw(&factory),
                outputstream.param().abi(),
                &mut result__,
            )
            .and_then(|| Type::from_abi(result__))
        }
    }

    pub(super) fn data_reader<P0>(inputstream: P0) -> Result<DataReader>
    where
        P0: Param<IInputStream>,
    {
        let factory = factory::<DataReader, IDataReaderFactory>()?;
        // Safety: this is the generated reader invocation with an owned scoped
        // factory, identical typed parameter ABI and checked ownership transfer.
        unsafe {
            let mut result__ = core::mem::zeroed();
            (Interface::vtable(&factory).CreateDataReader)(
                Interface::as_raw(&factory),
                inputstream.param().abi(),
                &mut result__,
            )
            .and_then(|| Type::from_abi(result__))
        }
    }

    pub(super) fn load_document<P0>(
        inputstream: P0,
    ) -> Result<windows_future::IAsyncOperation<PdfDocument>>
    where
        P0: Param<IRandomAccessStream>,
    {
        let factory = factory::<PdfDocument, IPdfDocumentStatics>()?;
        // Safety: the pinned generated stream ABI and async-operation output
        // conversion are unchanged. The caller retains its original join/order.
        unsafe {
            let mut result__ = core::mem::zeroed();
            (Interface::vtable(&factory).LoadFromStreamAsync)(
                Interface::as_raw(&factory),
                inputstream.param().abi(),
                &mut result__,
            )
            .and_then(|| Type::from_abi(result__))
        }
    }

    pub(super) fn render_options() -> Result<PdfPageRenderOptions> {
        let factory = factory::<PdfPageRenderOptions, IActivationFactory>()?;
        // Safety: the fixed public activation interface creates an owned
        // IInspectable; the normal typed cast preserves the generated sequence.
        unsafe { factory.ActivateInstance() }.and_then(|instance| instance.cast())
    }

    pub(super) fn png_encoder_id() -> Result<GUID> {
        let factory = factory::<BitmapEncoder, IBitmapEncoderStatics>()?;
        // Safety: the pinned generated method writes a GUID value, not an
        // interface. Only a successful HRESULT exposes that initialized output.
        unsafe {
            let mut result__ = core::mem::zeroed();
            (Interface::vtable(&factory).PngEncoderId)(Interface::as_raw(&factory), &mut result__)
                .map(|| result__)
        }
    }
}
// P10 SCOPED FACTORIES END

// P9 TEST-ONLY BEGIN
#[cfg(all(test, windows))]
mod pdf_test_diagnostic {
    use super::PdfFailureStage;
    use std::cell::Cell;
    use std::io::Write;
    use std::marker::PhantomData;

    #[derive(Clone, Copy)]
    enum Phase {
        Valid,
        Limit,
    }

    #[derive(Clone, Copy)]
    pub(super) enum Edge {
        Enter,
        Leave,
    }

    #[derive(Clone, Copy)]
    pub(super) enum Case {
        ValidBegin,
        ValidEnd,
        AssertionsDone,
        LimitBegin,
        LimitEnd,
        Complete,
    }

    #[derive(Clone, Copy)]
    pub(super) enum Teardown {
        BodyComplete,
        UninitEnter,
        UninitLeave,
    }

    enum Marker {
        Operation(PdfFailureStage, Edge),
        Case(Case),
        Teardown(Teardown),
    }

    std::thread_local! {
        static PHASE: Cell<Option<Phase>> = const { Cell::new(None) };
    }

    // Finite static byte strings only: no input, stage number or error formatting.
    const OPERATIONS: [[[&[u8]; 2]; 18]; 2] = [
        [
            [
                b"ASTER_PDF_DIAG_V1|valid|64|enter\n",
                b"ASTER_PDF_DIAG_V1|valid|64|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|valid|65|enter\n",
                b"ASTER_PDF_DIAG_V1|valid|65|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|valid|66|enter\n",
                b"ASTER_PDF_DIAG_V1|valid|66|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|valid|67|enter\n",
                b"ASTER_PDF_DIAG_V1|valid|67|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|valid|68|enter\n",
                b"ASTER_PDF_DIAG_V1|valid|68|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|valid|69|enter\n",
                b"ASTER_PDF_DIAG_V1|valid|69|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|valid|70|enter\n",
                b"ASTER_PDF_DIAG_V1|valid|70|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|valid|71|enter\n",
                b"ASTER_PDF_DIAG_V1|valid|71|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|valid|72|enter\n",
                b"ASTER_PDF_DIAG_V1|valid|72|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|valid|73|enter\n",
                b"ASTER_PDF_DIAG_V1|valid|73|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|valid|74|enter\n",
                b"ASTER_PDF_DIAG_V1|valid|74|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|valid|75|enter\n",
                b"ASTER_PDF_DIAG_V1|valid|75|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|valid|76|enter\n",
                b"ASTER_PDF_DIAG_V1|valid|76|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|valid|77|enter\n",
                b"ASTER_PDF_DIAG_V1|valid|77|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|valid|78|enter\n",
                b"ASTER_PDF_DIAG_V1|valid|78|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|valid|79|enter\n",
                b"ASTER_PDF_DIAG_V1|valid|79|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|valid|80|enter\n",
                b"ASTER_PDF_DIAG_V1|valid|80|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|valid|81|enter\n",
                b"ASTER_PDF_DIAG_V1|valid|81|leave\n",
            ],
        ],
        [
            [
                b"ASTER_PDF_DIAG_V1|limit|64|enter\n",
                b"ASTER_PDF_DIAG_V1|limit|64|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|limit|65|enter\n",
                b"ASTER_PDF_DIAG_V1|limit|65|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|limit|66|enter\n",
                b"ASTER_PDF_DIAG_V1|limit|66|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|limit|67|enter\n",
                b"ASTER_PDF_DIAG_V1|limit|67|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|limit|68|enter\n",
                b"ASTER_PDF_DIAG_V1|limit|68|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|limit|69|enter\n",
                b"ASTER_PDF_DIAG_V1|limit|69|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|limit|70|enter\n",
                b"ASTER_PDF_DIAG_V1|limit|70|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|limit|71|enter\n",
                b"ASTER_PDF_DIAG_V1|limit|71|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|limit|72|enter\n",
                b"ASTER_PDF_DIAG_V1|limit|72|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|limit|73|enter\n",
                b"ASTER_PDF_DIAG_V1|limit|73|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|limit|74|enter\n",
                b"ASTER_PDF_DIAG_V1|limit|74|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|limit|75|enter\n",
                b"ASTER_PDF_DIAG_V1|limit|75|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|limit|76|enter\n",
                b"ASTER_PDF_DIAG_V1|limit|76|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|limit|77|enter\n",
                b"ASTER_PDF_DIAG_V1|limit|77|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|limit|78|enter\n",
                b"ASTER_PDF_DIAG_V1|limit|78|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|limit|79|enter\n",
                b"ASTER_PDF_DIAG_V1|limit|79|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|limit|80|enter\n",
                b"ASTER_PDF_DIAG_V1|limit|80|leave\n",
            ],
            [
                b"ASTER_PDF_DIAG_V1|limit|81|enter\n",
                b"ASTER_PDF_DIAG_V1|limit|81|leave\n",
            ],
        ],
    ];

    pub(super) struct PhaseGuard {
        previous: Option<Phase>,
        // The activation belongs to this test thread and cannot cross threads.
        _thread_bound: PhantomData<*mut ()>,
    }

    impl PhaseGuard {
        pub(super) fn valid() -> Self {
            Self {
                previous: PHASE.with(|phase| phase.replace(Some(Phase::Valid))),
                _thread_bound: PhantomData,
            }
        }

        pub(super) fn limit(&self) {
            PHASE.with(|phase| phase.set(Some(Phase::Limit)));
        }
    }

    impl Drop for PhaseGuard {
        fn drop(&mut self) {
            PHASE.with(|phase| phase.set(self.previous));
        }
    }

    fn emit(marker: Marker) {
        PHASE.with(|phase| {
            let Some(phase) = phase.get() else {
                return;
            };
            let bytes: &[u8] = match marker {
                Marker::Operation(stage, edge) => {
                    let phase_index = match phase {
                        Phase::Valid => 0,
                        Phase::Limit => 1,
                    };
                    let edge_index = match edge {
                        Edge::Enter => 0,
                        Edge::Leave => 1,
                    };
                    OPERATIONS[phase_index][stage as usize - 64][edge_index]
                }
                Marker::Case(case) => match case {
                    Case::ValidBegin => b"ASTER_PDF_DIAG_V1|valid_begin\n",
                    Case::ValidEnd => b"ASTER_PDF_DIAG_V1|valid_end\n",
                    Case::AssertionsDone => b"ASTER_PDF_DIAG_V1|assertions_done\n",
                    Case::LimitBegin => b"ASTER_PDF_DIAG_V1|limit_begin\n",
                    Case::LimitEnd => b"ASTER_PDF_DIAG_V1|limit_end\n",
                    Case::Complete => b"ASTER_PDF_DIAG_V1|complete\n",
                },
                Marker::Teardown(teardown) => match (phase, teardown) {
                    (Phase::Valid, Teardown::BodyComplete) => {
                        b"ASTER_PDF_DIAG_V1|valid|body_complete\n"
                    }
                    (Phase::Valid, Teardown::UninitEnter) => {
                        b"ASTER_PDF_DIAG_V1|valid|uninit_enter\n"
                    }
                    (Phase::Valid, Teardown::UninitLeave) => {
                        b"ASTER_PDF_DIAG_V1|valid|uninit_leave\n"
                    }
                    (Phase::Limit, Teardown::BodyComplete) => {
                        b"ASTER_PDF_DIAG_V1|limit|body_complete\n"
                    }
                    (Phase::Limit, Teardown::UninitEnter) => {
                        b"ASTER_PDF_DIAG_V1|limit|uninit_enter\n"
                    }
                    (Phase::Limit, Teardown::UninitLeave) => {
                        b"ASTER_PDF_DIAG_V1|limit|uninit_leave\n"
                    }
                },
            };
            // Direct stderr bypasses libtest's capture buffer. A write failure
            // leaves an incomplete trace; it never changes a processing result.
            let mut stderr = std::io::stderr().lock();
            let _ = stderr.write_all(bytes);
            let _ = stderr.flush();
        });
    }

    pub(super) fn operation(stage: PdfFailureStage, edge: Edge) {
        emit(Marker::Operation(stage, edge));
    }

    pub(super) fn case(case: Case) {
        emit(Marker::Case(case));
    }

    pub(super) fn teardown(teardown: Teardown) {
        emit(Marker::Teardown(teardown));
    }
}
// P9 TEST-ONLY END
#[cfg(windows)]
fn render_pdf(bytes: &[u8]) -> Result<Vec<ProcessedPage>, ProcessingError> {
    #[link(name = "runtimeobject")]
    unsafe extern "system" {
        fn RoInitialize(init_type: u32) -> i32;
        fn RoUninitialize();
    }
    struct Apartment;
    impl Drop for Apartment {
        fn drop(&mut self) {
            // P9 TEST-ONLY BEGIN
            #[cfg(all(test, windows))]
            pdf_test_diagnostic::teardown(pdf_test_diagnostic::Teardown::UninitEnter);
            // P9 TEST-ONLY END
            unsafe {
                RoUninitialize();
            }
            // P9 TEST-ONLY BEGIN
            #[cfg(all(test, windows))]
            pdf_test_diagnostic::teardown(pdf_test_diagnostic::Teardown::UninitLeave);
            // P9 TEST-ONLY END
        }
    }
    // Dedicated child thread, before any Tauri/native app bootstrap. No STA pump.
    // P9 TEST-ONLY BEGIN
    #[cfg(all(test, windows))]
    pdf_test_diagnostic::operation(
        PdfFailureStage::InitialiseRuntime,
        pdf_test_diagnostic::Edge::Enter,
    );
    // P9 TEST-ONLY END
    if unsafe { RoInitialize(1) } < 0 {
        return Err(ProcessingError::Pdf(PdfFailureStage::InitialiseRuntime));
    }
    // P9 TEST-ONLY BEGIN
    #[cfg(all(test, windows))]
    pdf_test_diagnostic::operation(
        PdfFailureStage::InitialiseRuntime,
        pdf_test_diagnostic::Edge::Leave,
    );
    // P9 TEST-ONLY END
    let _apartment = Apartment;
    let fail = |stage| move |_: windows::core::Error| ProcessingError::Pdf(stage);
    // P9 TEST-ONLY BEGIN
    #[cfg(all(test, windows))]
    pdf_test_diagnostic::operation(
        PdfFailureStage::CreateInputStream,
        pdf_test_diagnostic::Edge::Enter,
    );
    // P9 TEST-ONLY END
    let stream = new_memory_stream().map_err(fail(PdfFailureStage::CreateInputStream))?;
    // P9 TEST-ONLY BEGIN
    #[cfg(all(test, windows))]
    pdf_test_diagnostic::operation(
        PdfFailureStage::CreateInputStream,
        pdf_test_diagnostic::Edge::Leave,
    );
    // P9 TEST-ONLY END
    // P9 TEST-ONLY BEGIN
    #[cfg(all(test, windows))]
    pdf_test_diagnostic::operation(
        PdfFailureStage::CreateInputWriter,
        pdf_test_diagnostic::Edge::Enter,
    );
    // P9 TEST-ONLY END
    let writer = scoped_pdf_factories::data_writer(&stream)
        .map_err(fail(PdfFailureStage::CreateInputWriter))?;
    // P9 TEST-ONLY BEGIN
    #[cfg(all(test, windows))]
    pdf_test_diagnostic::operation(
        PdfFailureStage::CreateInputWriter,
        pdf_test_diagnostic::Edge::Leave,
    );
    // P9 TEST-ONLY END
    // P9 TEST-ONLY BEGIN
    #[cfg(all(test, windows))]
    pdf_test_diagnostic::operation(
        PdfFailureStage::WriteInput,
        pdf_test_diagnostic::Edge::Enter,
    );
    // P9 TEST-ONLY END
    writer
        .WriteBytes(bytes)
        .map_err(fail(PdfFailureStage::WriteInput))?;
    // P9 TEST-ONLY BEGIN
    #[cfg(all(test, windows))]
    pdf_test_diagnostic::operation(
        PdfFailureStage::WriteInput,
        pdf_test_diagnostic::Edge::Leave,
    );
    // P9 TEST-ONLY END
    // P9 TEST-ONLY BEGIN
    #[cfg(all(test, windows))]
    pdf_test_diagnostic::operation(
        PdfFailureStage::StoreInput,
        pdf_test_diagnostic::Edge::Enter,
    );
    // P9 TEST-ONLY END
    if writer
        .StoreAsync()
        .map_err(fail(PdfFailureStage::StoreInput))?
        .join()
        .map_err(fail(PdfFailureStage::StoreInput))? as usize
        != bytes.len()
    {
        return Err(ProcessingError::Invalid);
    }
    // P9 TEST-ONLY BEGIN
    #[cfg(all(test, windows))]
    pdf_test_diagnostic::operation(
        PdfFailureStage::StoreInput,
        pdf_test_diagnostic::Edge::Leave,
    );
    // P9 TEST-ONLY END
    // P9 TEST-ONLY BEGIN
    #[cfg(all(test, windows))]
    pdf_test_diagnostic::operation(
        PdfFailureStage::DetachInput,
        pdf_test_diagnostic::Edge::Enter,
    );
    // P9 TEST-ONLY END
    writer
        .DetachStream()
        .map_err(fail(PdfFailureStage::DetachInput))?;
    // P9 TEST-ONLY BEGIN
    #[cfg(all(test, windows))]
    pdf_test_diagnostic::operation(
        PdfFailureStage::DetachInput,
        pdf_test_diagnostic::Edge::Leave,
    );
    // P9 TEST-ONLY END
    // P9 TEST-ONLY BEGIN
    #[cfg(all(test, windows))]
    pdf_test_diagnostic::operation(PdfFailureStage::SeekInput, pdf_test_diagnostic::Edge::Enter);
    // P9 TEST-ONLY END
    stream.Seek(0).map_err(fail(PdfFailureStage::SeekInput))?;
    // P9 TEST-ONLY BEGIN
    #[cfg(all(test, windows))]
    pdf_test_diagnostic::operation(PdfFailureStage::SeekInput, pdf_test_diagnostic::Edge::Leave);
    // P9 TEST-ONLY END
    // P9 TEST-ONLY BEGIN
    #[cfg(all(test, windows))]
    pdf_test_diagnostic::operation(
        PdfFailureStage::LoadDocument,
        pdf_test_diagnostic::Edge::Enter,
    );
    // P9 TEST-ONLY END
    let document = scoped_pdf_factories::load_document(&stream)
        .map_err(fail(PdfFailureStage::LoadDocument))?
        .join()
        .map_err(fail(PdfFailureStage::LoadDocument))?;
    // P9 TEST-ONLY BEGIN
    #[cfg(all(test, windows))]
    pdf_test_diagnostic::operation(
        PdfFailureStage::LoadDocument,
        pdf_test_diagnostic::Edge::Leave,
    );
    // P9 TEST-ONLY END
    // P9 TEST-ONLY BEGIN
    #[cfg(all(test, windows))]
    pdf_test_diagnostic::operation(
        PdfFailureStage::InspectDocument,
        pdf_test_diagnostic::Edge::Enter,
    );
    // P9 TEST-ONLY END
    if document
        .IsPasswordProtected()
        .map_err(fail(PdfFailureStage::InspectDocument))?
    {
        return Err(ProcessingError::Unsupported);
    }
    let count = document
        .PageCount()
        .map_err(fail(PdfFailureStage::InspectDocument))? as usize;
    // P9 TEST-ONLY BEGIN
    #[cfg(all(test, windows))]
    pdf_test_diagnostic::operation(
        PdfFailureStage::InspectDocument,
        pdf_test_diagnostic::Edge::Leave,
    );
    // P9 TEST-ONLY END
    if count == 0 || count > MAX_PDF_PAGES {
        return Err(ProcessingError::Limit);
    }
    let mut pages = Vec::with_capacity(count);
    for index in 0..count {
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::operation(PdfFailureStage::ReadPage, pdf_test_diagnostic::Edge::Enter);
        // P9 TEST-ONLY END
        let page = document
            .GetPage(index as u32)
            .map_err(fail(PdfFailureStage::ReadPage))?;
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::operation(PdfFailureStage::ReadPage, pdf_test_diagnostic::Edge::Leave);
        // P9 TEST-ONLY END
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::operation(
            PdfFailureStage::ReadPageSize,
            pdf_test_diagnostic::Edge::Enter,
        );
        // P9 TEST-ONLY END
        let size = page.Size().map_err(fail(PdfFailureStage::ReadPageSize))?;
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::operation(
            PdfFailureStage::ReadPageSize,
            pdf_test_diagnostic::Edge::Leave,
        );
        // P9 TEST-ONLY END
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
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::operation(
            PdfFailureStage::CreateRenderOptions,
            pdf_test_diagnostic::Edge::Enter,
        );
        // P9 TEST-ONLY END
        let options = scoped_pdf_factories::render_options()
            .map_err(fail(PdfFailureStage::CreateRenderOptions))?;
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::operation(
            PdfFailureStage::CreateRenderOptions,
            pdf_test_diagnostic::Edge::Leave,
        );
        // P9 TEST-ONLY END
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::operation(
            PdfFailureStage::ConfigureRender,
            pdf_test_diagnostic::Edge::Enter,
        );
        // P9 TEST-ONLY END
        options
            .SetBitmapEncoderId(
                scoped_pdf_factories::png_encoder_id()
                    .map_err(fail(PdfFailureStage::ConfigureRender))?,
            )
            .map_err(fail(PdfFailureStage::ConfigureRender))?;
        options
            .SetIsIgnoringHighContrast(true)
            .map_err(fail(PdfFailureStage::ConfigureRender))?;
        options
            .SetDestinationWidth(width)
            .map_err(fail(PdfFailureStage::ConfigureRender))?;
        options
            .SetDestinationHeight(height)
            .map_err(fail(PdfFailureStage::ConfigureRender))?;
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::operation(
            PdfFailureStage::ConfigureRender,
            pdf_test_diagnostic::Edge::Leave,
        );
        // P9 TEST-ONLY END
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::operation(
            PdfFailureStage::CreateOutputStream,
            pdf_test_diagnostic::Edge::Enter,
        );
        // P9 TEST-ONLY END
        let output = new_memory_stream().map_err(fail(PdfFailureStage::CreateOutputStream))?;
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::operation(
            PdfFailureStage::CreateOutputStream,
            pdf_test_diagnostic::Edge::Leave,
        );
        // P9 TEST-ONLY END
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::operation(
            PdfFailureStage::RenderPage,
            pdf_test_diagnostic::Edge::Enter,
        );
        // P9 TEST-ONLY END
        page.RenderWithOptionsToStreamAsync(&output, &options)
            .map_err(fail(PdfFailureStage::RenderPage))?
            .join()
            .map_err(fail(PdfFailureStage::RenderPage))?;
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::operation(
            PdfFailureStage::RenderPage,
            pdf_test_diagnostic::Edge::Leave,
        );
        // P9 TEST-ONLY END
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::operation(
            PdfFailureStage::ReadOutput,
            pdf_test_diagnostic::Edge::Enter,
        );
        // P9 TEST-ONLY END
        let length = usize::try_from(output.Size().map_err(fail(PdfFailureStage::ReadOutput))?)
            .map_err(|_| ProcessingError::Limit)?;
        if length == 0 || length > MAX_SOURCE_BYTES {
            return Err(ProcessingError::Limit);
        }
        let input = output
            .GetInputStreamAt(0)
            .map_err(fail(PdfFailureStage::ReadOutput))?;
        let reader =
            scoped_pdf_factories::data_reader(&input).map_err(fail(PdfFailureStage::ReadOutput))?;
        if reader
            .LoadAsync(length as u32)
            .map_err(fail(PdfFailureStage::ReadOutput))?
            .join()
            .map_err(fail(PdfFailureStage::ReadOutput))? as usize
            != length
        {
            return Err(ProcessingError::Invalid);
        }
        let mut rendered = vec![0; length];
        reader
            .ReadBytes(&mut rendered)
            .map_err(fail(PdfFailureStage::ReadOutput))?;
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::operation(
            PdfFailureStage::ReadOutput,
            pdf_test_diagnostic::Edge::Leave,
        );
        // P9 TEST-ONLY END
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::operation(
            PdfFailureStage::NormalisePage,
            pdf_test_diagnostic::Edge::Enter,
        );
        // P9 TEST-ONLY END
        let normalised =
            decode_static_image(InputKind::Png, &rendered).map_err(pdf_normalisation_error)?;
        if normalised.width != width || normalised.height != height {
            return Err(ProcessingError::Invalid);
        }
        pages.push(normalised);
        validate_pages(&pages)?;
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::operation(
            PdfFailureStage::NormalisePage,
            pdf_test_diagnostic::Edge::Leave,
        );
        // P9 TEST-ONLY END
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::operation(
            PdfFailureStage::ClosePage,
            pdf_test_diagnostic::Edge::Enter,
        );
        // P9 TEST-ONLY END
        page.Close().map_err(fail(PdfFailureStage::ClosePage))?;
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::operation(
            PdfFailureStage::ClosePage,
            pdf_test_diagnostic::Edge::Leave,
        );
        // P9 TEST-ONLY END
    }
    if pages.len() != count {
        return Err(ProcessingError::Invalid);
    }
    // P9 TEST-ONLY BEGIN
    #[cfg(all(test, windows))]
    pdf_test_diagnostic::teardown(pdf_test_diagnostic::Teardown::BodyComplete);
    // P9 TEST-ONLY END
    Ok(pages)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalisation_diagnostics_preserve_limit_and_boundary_errors() {
        for error in [
            ProcessingError::Invalid,
            ProcessingError::Unsupported,
            ProcessingError::Worker,
        ] {
            assert_eq!(
                pdf_normalisation_error(error),
                ProcessingError::Pdf(PdfFailureStage::NormalisePage)
            );
        }
        for error in [
            ProcessingError::Limit,
            ProcessingError::Isolation,
            ProcessingError::Cancelled,
            ProcessingError::Timeout,
            ProcessingError::Pdf(PdfFailureStage::ReadOutput),
        ] {
            assert_eq!(pdf_normalisation_error(error), error);
        }
    }

    #[test]
    fn pdf_failure_stages_are_closed_bounded_and_never_success() {
        for stage in PdfFailureStage::ALL {
            let code = stage.worker_exit_code();
            assert!((64..=81).contains(&code));
            assert_eq!(PdfFailureStage::from_worker_exit(code), Some(stage));
            let reason = stage.diagnostic_reason();
            assert!(reason.len() <= 32);
            assert!(reason.starts_with("pdf-"));
            assert!(reason.chars().all(|c| c.is_ascii_lowercase() || c == '-'));
        }
        for code in [0, 1, 3, 63, 82, 255, u32::MAX] {
            assert_eq!(PdfFailureStage::from_worker_exit(code), None);
        }
    }

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
    fn complete_png_rejects_missing_truncated_corrupt_iend_and_suffixes() {
        let canonical = page_from_rgba(2, 1, vec![10, 20, 30, 255, 40, 50, 60, 255])
            .unwrap()
            .png;
        let iend = canonical.len() - 12;
        assert_eq!(&canonical[iend + 4..iend + 8], b"IEND");
        let mut wrong_crc = canonical.clone();
        *wrong_crc.last_mut().unwrap() ^= 1;
        let mut garbage = canonical.clone();
        garbage.extend_from_slice(b"synthetic-suffix");
        let mut concatenated = canonical.clone();
        concatenated.extend_from_slice(&canonical);
        let mut unknown_critical = canonical.clone();
        unknown_critical.splice(iend..iend, png_chunk(b"ABCD", b""));
        let mut bad_ancillary_crc = png_chunk(b"tEXt", b"Label\0synthetic");
        *bad_ancillary_crc.last_mut().unwrap() ^= 1;
        let mut corrupt_ancillary = canonical.clone();
        corrupt_ancillary.splice(iend..iend, bad_ancillary_crc);
        let cases = [
            canonical[..iend].to_vec(),
            canonical[..canonical.len() - 4].to_vec(),
            canonical[..canonical.len() - 1].to_vec(),
            wrong_crc,
            garbage,
            concatenated,
            unknown_critical,
            corrupt_ancillary,
        ];
        for (index, source) in cases.iter().enumerate() {
            assert!(
                process_in_worker(InputKind::Png, source).is_err(),
                "whole PNG case {index} must not admit intact pixels from an incomplete container"
            );
        }
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
    fn complete_gif_refuses_screen_clipping_missing_end_codes_and_suffixes() {
        let mut canonical = Vec::new();
        image::codecs::gif::GifEncoder::new(&mut canonical)
            .encode_frame(image::Frame::new(image::RgbaImage::from_pixel(
                3,
                2,
                image::Rgba([20, 80, 130, 255]),
            )))
            .unwrap();
        assert_eq!(canonical.last(), Some(&b';'));
        let mut smaller_screen = canonical.clone();
        smaller_screen[6..8].copy_from_slice(&1u16.to_le_bytes());
        smaller_screen[8..10].copy_from_slice(&1u16.to_le_bytes());
        let descriptor = canonical.iter().position(|byte| *byte == 0x2c).unwrap();
        assert!(descriptor >= 13);
        let mut outside_screen = canonical.clone();
        outside_screen[descriptor + 1..descriptor + 3].copy_from_slice(&3u16.to_le_bytes());
        let mut garbage = canonical.clone();
        garbage.extend_from_slice(b"synthetic-suffix;");
        let mut concatenated = canonical.clone();
        concatenated.extend_from_slice(&canonical);
        // One decoded black pixel but no LZW end-of-information code. The
        // default permissive codec can finish the pixel; strict validation must
        // still deny its incomplete image-data stream.
        let missing_lzw_end = b"GIF89a\x01\x00\x01\x00\x80\x00\x00\x00\x00\x00\xff\xff\xff\x2c\x00\x00\x00\x00\x01\x00\x01\x00\x00\x02\x01\x04\x00;".to_vec();
        let cases = [
            smaller_screen,
            outside_screen,
            canonical[..canonical.len() - 1].to_vec(),
            missing_lzw_end,
            garbage,
            concatenated,
        ];
        for (index, source) in cases.iter().enumerate() {
            assert!(
                process_in_worker(InputKind::Gif, source).is_err(),
                "whole GIF case {index} must not admit clipping or incomplete content"
            );
        }
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
    fn complete_jpeg_refuses_missing_eoi_partial_entropy_and_suffixes() {
        let mut canonical = Vec::new();
        let pixels: Vec<_> = (0..32 * 32 * 3)
            .map(|index| ((index * 37 + index / 17) % 256) as u8)
            .collect();
        image::codecs::jpeg::JpegEncoder::new(&mut canonical)
            .encode(&pixels, 32, 32, image::ExtendedColorType::Rgb8)
            .unwrap();
        assert!(canonical.ends_with(b"\xff\xd9"));
        let scan = canonical
            .windows(2)
            .position(|marker| marker == b"\xff\xda")
            .unwrap();
        let header_length = u16::from_be_bytes([canonical[scan + 2], canonical[scan + 3]]) as usize;
        let entropy = scan + 2 + header_length;
        let entropy_length = canonical.len() - 2 - entropy;
        assert!(entropy_length > 4);
        let mut cases = vec![
            canonical[..canonical.len() - 2].to_vec(),
            canonical[..canonical.len() - 1].to_vec(),
        ];
        for kept in [0, 1, entropy_length / 2] {
            let mut partial = canonical[..entropy + kept].to_vec();
            // A forged terminal marker must not turn partial entropy into a
            // successful grey/default-filled normalised image.
            partial.extend_from_slice(b"\xff\xd9");
            cases.push(partial);
        }
        let mut garbage = canonical.clone();
        garbage.extend_from_slice(b"synthetic-suffix\xff\xd9");
        cases.push(garbage);
        let mut concatenated = canonical.clone();
        concatenated.extend_from_slice(&canonical);
        cases.push(concatenated);
        for (index, source) in cases.iter().enumerate() {
            assert!(
                process_in_worker(InputKind::Jpeg, source).is_err(),
                "whole JPEG case {index} must not admit a partial image"
            );
        }
    }

    #[test]
    fn valid_short_baseline_and_progressive_jpeg_keep_all_pixels() {
        use base64::Engine;

        // Pillow 12.3.0 generated synthetic RGB fixtures in memory only,
        // quality=90/progressive=true; no user images or live parser execution.
        const TINY_PROGRESSIVE: &str = "/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAMCAgMCAgMDAwMEAwMEBQgFBQQEBQoHBwYIDAoMDAsKCwsNDhIQDQ4RDgsLEBYQERMUFRUVDA8XGBYUGBIUFRT/2wBDAQMEBAUEBQkFBQkUDQsNFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBT/wgARCAACAAMDASIAAhEBAxEB/8QAFQABAQAAAAAAAAAAAAAAAAAAAAf/xAAUAQEAAAAAAAAAAAAAAAAAAAAG/9oADAMBAAIQAxAAAAGXhcR//8QAFBABAAAAAAAAAAAAAAAAAAAAAP/aAAgBAQABBQJ//8QAFBEBAAAAAAAAAAAAAAAAAAAAAP/aAAgBAwEBPwF//8QAFBEBAAAAAAAAAAAAAAAAAAAAAP/aAAgBAgEBPwF//8QAFBABAAAAAAAAAAAAAAAAAAAAAP/aAAgBAQAGPwJ//8QAFBABAAAAAAAAAAAAAAAAAAAAAP/aAAgBAQABPyF//9oADAMBAAIAAwAAABAP/8QAFBEBAAAAAAAAAAAAAAAAAAAAAP/aAAgBAwEBPxB//8QAFBEBAAAAAAAAAAAAAAAAAAAAAP/aAAgBAgEBPxB//8QAFBABAAAAAAAAAAAAAAAAAAAAAP/aAAgBAQABPxB//9k=";
        const CHECKER_PROGRESSIVE: &str = "/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAMCAgMCAgMDAwMEAwMEBQgFBQQEBQoHBwYIDAoMDAsKCwsNDhIQDQ4RDgsLEBYQERMUFRUVDA8XGBYUGBIUFRT/2wBDAQMEBAUEBQkFBQkUDQsNFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBT/wgARCAAQABADASIAAhEBAxEB/8QAFQABAQAAAAAAAAAAAAAAAAAAAAb/xAAVAQEBAAAAAAAAAAAAAAAAAAAFBv/aAAwDAQACEAMQAAABlxTM/wD/xAAUEAEAAAAAAAAAAAAAAAAAAAAg/9oACAEBAAEFAh//xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oACAEDAQE/AX//xAAUEQEAAAAAAAAAAAAAAAAAAAAA/9oACAECAQE/AX//xAAUEAEAAAAAAAAAAAAAAAAAAAAg/9oACAEBAAY/Ah//xAAVEAEBAAAAAAAAAAAAAAAAAADw4f/aAAgBAQABPyEUUUX/2gAMAwEAAgADAAAAEP8A/8QAFBEBAAAAAAAAAAAAAAAAAAAAAP/aAAgBAwEBPxB//8QAFBEBAAAAAAAAAAAAAAAAAAAAAP/aAAgBAgEBPxB//8QAGxAAAgEFAAAAAAAAAAAAAAAAETFhAEFRgbH/2gAIAQEAAT8QcnZ7mXc1cnZ7mXc1cnZ7mXc1cnZ7mXc1/9k=";
        let mut cases = vec![
            (
                3,
                2,
                base64::engine::general_purpose::STANDARD
                    .decode(TINY_PROGRESSIVE)
                    .unwrap(),
            ),
            (
                16,
                16,
                base64::engine::general_purpose::STANDARD
                    .decode(CHECKER_PROGRESSIVE)
                    .unwrap(),
            ),
        ];
        for (width, height) in [(1, 1), (3, 2), (32, 32)] {
            let mut jpeg = Vec::new();
            image::codecs::jpeg::JpegEncoder::new(&mut jpeg)
                .encode(
                    &vec![50u8; width as usize * height as usize * 3],
                    width,
                    height,
                    image::ExtendedColorType::Rgb8,
                )
                .unwrap();
            cases.push((width, height, jpeg));
        }
        for (width, height, source) in cases {
            let ProcessedContent::Pages(pages) =
                process_in_worker(InputKind::Jpeg, &source).unwrap().content
            else {
                panic!("pages")
            };
            assert_eq!(pages.len(), 1);
            assert_eq!((pages[0].width, pages[0].height), (width, height));
            assert_eq!(pages[0].rgba.len(), width as usize * height as usize * 4);
            if (width, height) == (16, 16) {
                // The synthetic checker has alternating luminance in every
                // row. 4:2:0 chroma subsampling may blend its original RGB,
                // but neither a blank row nor default fill may pass.
                for row in pages[0].rgba.chunks_exact(width as usize * 4) {
                    for pair in row.chunks_exact(8) {
                        let contrast: u32 = (0..3)
                            .map(|channel| u32::from(pair[channel].abs_diff(pair[channel + 4])))
                            .sum();
                        assert!(contrast > 12, "missing progressive checker content");
                    }
                }
            } else {
                let expected = if source.windows(2).any(|marker| marker == b"\xff\xc2") {
                    [20u8, 80, 130]
                } else {
                    [50u8; 3]
                };
                for pixel in pages[0].rgba.chunks_exact(4) {
                    assert_eq!(pixel[3], 255);
                    for channel in 0..3 {
                        assert!(
                            pixel[channel].abs_diff(expected[channel]) <= 5,
                            "missing flat synthetic JPEG content"
                        );
                    }
                }
            }
            let scan = source
                .windows(2)
                .position(|marker| marker == b"\xff\xda")
                .unwrap();
            let length = u16::from_be_bytes([source[scan + 2], source[scan + 3]]) as usize;
            let mut zero_entropy = source[..scan + 2 + length].to_vec();
            zero_entropy.extend_from_slice(b"\xff\xd9");
            assert!(process_in_worker(InputKind::Jpeg, &zero_entropy).is_err());
        }
    }

    #[test]
    fn jpeg_restart_segments_require_their_nonfinal_marker() {
        use base64::Engine;

        // Pillow 12.3.0: in-memory RGB 32x16 flat (20,80,130), quality90,
        // subsampling=0, restart_marker_blocks=1. Eight real MCUs, seven RSTs.
        const RESTART_JPEG: &str = "/9j/4AAQSkZJRgABAQAAAQABAAD/2wBDAAMCAgMCAgMDAwMEAwMEBQgFBQQEBQoHBwYIDAoMDAsKCwsNDhIQDQ4RDgsLEBYQERMUFRUVDA8XGBYUGBIUFRT/2wBDAQMEBAUEBQkFBQkUDQsNFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBT/wAARCAAQACADAREAAhEBAxEB/8QAHwAAAQUBAQEBAQEAAAAAAAAAAAECAwQFBgcICQoL/8QAtRAAAgEDAwIEAwUFBAQAAAF9AQIDAAQRBRIhMUEGE1FhByJxFDKBkaEII0KxwRVS0fAkM2JyggkKFhcYGRolJicoKSo0NTY3ODk6Q0RFRkdISUpTVFVWV1hZWmNkZWZnaGlqc3R1dnd4eXqDhIWGh4iJipKTlJWWl5iZmqKjpKWmp6ipqrKztLW2t7i5usLDxMXGx8jJytLT1NXW19jZ2uHi4+Tl5ufo6erx8vP09fb3+Pn6/8QAHwEAAwEBAQEBAQEBAQAAAAAAAAECAwQFBgcICQoL/8QAtREAAgECBAQDBAcFBAQAAQJ3AAECAxEEBSExBhJBUQdhcRMiMoEIFEKRobHBCSMzUvAVYnLRChYkNOEl8RcYGRomJygpKjU2Nzg5OkNERUZHSElKU1RVVldYWVpjZGVmZ2hpanN0dXZ3eHl6goOEhYaHiImKkpOUlZaXmJmaoqOkpaanqKmqsrO0tba3uLm6wsPExcbHyMnK0tPU1dbX2Nna4uPk5ebn6Onq8vP09fb3+Pn6/90ABAAB/9oADAMBAAIRAxEAPwD5fr9dPyQ//9D5fr9dPyQ//9H5fr9dPyQ//9L5fr9dPyQ//9P5fr9dPyQ//9T5fr9dPyQ//9X5fr9dPyQ//9b5fr9dPyQ//9k=";
        let source = base64::engine::general_purpose::STANDARD
            .decode(RESTART_JPEG)
            .unwrap();
        let restarts: Vec<_> = source
            .windows(2)
            .enumerate()
            .filter_map(|(index, marker)| {
                (marker[0] == 0xff && (0xd0..=0xd7).contains(&marker[1])).then_some(index)
            })
            .collect();
        assert_eq!(restarts.len(), 7);
        let ProcessedContent::Pages(pages) =
            process_in_worker(InputKind::Jpeg, &source).unwrap().content
        else {
            panic!("pages")
        };
        assert_eq!(pages.len(), 1);
        assert_eq!((pages[0].width, pages[0].height), (32, 16));
        assert_eq!(pages[0].rgba.len(), 32 * 16 * 4);
        for pixel in pages[0].rgba.chunks_exact(4) {
            assert_eq!(pixel[3], 255);
            for (actual, expected) in pixel[..3].iter().zip([20u8, 80, 130]) {
                assert!(
                    actual.abs_diff(expected) <= 5,
                    "missing restart JPEG content"
                );
            }
        }
        let first = restarts[0];
        assert!(first < restarts[1]); // Not the final restart boundary.
        let mut missing_restart = source.clone();
        drop(missing_restart.drain(first..first + 2));
        assert!(
            process_in_worker(InputKind::Jpeg, &missing_restart).is_err(),
            "missing nonfinal restart must not concatenate entropy segments"
        );
        let mut garbage = source.clone();
        drop(garbage.splice(first..first, [0x12, 0x34]));
        assert!(
            process_in_worker(InputKind::Jpeg, &garbage).is_err(),
            "extra entropy before a restart must not be skipped"
        );
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
    #[test]
    fn memory_stream_capability_admission_denies_each_false_combination() {
        assert!(admit_memory_stream_capabilities(true, true).is_ok());
        for (can_read, can_write) in [(true, false), (false, true), (false, false)] {
            let error = admit_memory_stream_capabilities(can_read, can_write).unwrap_err();
            assert_eq!(error.code(), windows::Win32::Foundation::E_FAIL);
        }
    }

    #[cfg(windows)]
    #[test]
    fn documented_memory_stream_round_trip_clone_and_lifetime() {
        use windows::Win32::System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED};

        // This is a direct synthetic COM fixture, not a restricted-child PASS.
        unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.unwrap();
        struct Apartment;
        impl Drop for Apartment {
            fn drop(&mut self) {
                unsafe { RoUninitialize() };
            }
        }
        let _apartment = Apartment;

        // Measure the actual fixed synthetic backing; P7 did not record Stat.
        let backing = new_memory_backing_stream().unwrap();
        let mut stat = windows::Win32::System::Com::STATSTG::default();
        unsafe { backing.Stat(&mut stat, windows::Win32::System::Com::STATFLAG_NONAME) }.unwrap();
        assert!(stat.pwcsName.is_null());
        assert_eq!(
            stat.grfMode.0 & 3,
            windows::Win32::System::Com::STGM_READWRITE.0
        );
        assert_eq!(stat.cbSize, 0);
        drop(backing);

        let stream = new_memory_stream().unwrap();
        let independent = new_memory_stream().unwrap();
        for fresh in [&stream, &independent] {
            assert_eq!(fresh.Size().unwrap(), 0);
            assert_eq!(fresh.Position().unwrap(), 0);
            assert!(fresh.CanRead().unwrap());
            assert!(fresh.CanWrite().unwrap());
        }

        let expected = b"synthetic-memory-stream";
        let writer = scoped_pdf_factories::data_writer(&stream).unwrap();
        writer.WriteBytes(expected).unwrap();
        assert_eq!(
            writer.StoreAsync().unwrap().join().unwrap() as usize,
            expected.len()
        );
        writer.DetachStream().unwrap();
        assert_eq!(stream.Size().unwrap() as usize, expected.len());
        assert_eq!(independent.Size().unwrap(), 0);
        stream.Seek(0).unwrap();

        let cloned = stream.CloneStream().unwrap();
        assert_eq!(cloned.Position().unwrap(), 0);
        cloned.Seek(1).unwrap();
        assert_eq!(cloned.Position().unwrap(), 1);
        assert_eq!(stream.Position().unwrap(), 0);
        cloned.Seek(0).unwrap();
        drop(writer);
        drop(stream);

        // The clone and adapter must retain the backing allocation themselves.
        let input = cloned.GetInputStreamAt(0).unwrap();
        let reader = scoped_pdf_factories::data_reader(&input).unwrap();
        assert_eq!(
            reader
                .LoadAsync(expected.len() as u32)
                .unwrap()
                .join()
                .unwrap() as usize,
            expected.len()
        );
        let mut actual = vec![0; expected.len()];
        reader.ReadBytes(&mut actual).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(independent.Size().unwrap(), 0);
    }

    // P10 TWO-APARTMENT TEST BEGIN
    #[cfg(windows)]
    #[test]
    fn scoped_pdf_factories_survive_two_successive_apartments() {
        use windows::core::GUID;
        use windows::Win32::System::WinRT::{RoInitialize, RoUninitialize, RO_INIT_MULTITHREADED};

        struct Apartment;
        impl Drop for Apartment {
            fn drop(&mut self) {
                unsafe { RoUninitialize() };
            }
        }

        // Exactly two non-overlapping scopes on this ordinary test thread. Only
        // copyable GUID evidence crosses scopes; no COM factory/object does.
        let mut encoder_ids = [GUID::zeroed(); 2];
        for encoder_id in &mut encoder_ids {
            unsafe { RoInitialize(RO_INIT_MULTITHREADED) }.unwrap();
            let _apartment = Apartment;
            let stream = new_memory_stream().unwrap();
            let expected = b"synthetic-scoped-factory";
            let writer = scoped_pdf_factories::data_writer(&stream).unwrap();
            writer.WriteBytes(expected).unwrap();
            assert_eq!(
                writer.StoreAsync().unwrap().join().unwrap() as usize,
                expected.len()
            );
            writer.DetachStream().unwrap();
            assert_eq!(stream.Size().unwrap() as usize, expected.len());
            stream.Seek(0).unwrap();
            let input = stream.GetInputStreamAt(0).unwrap();
            let reader = scoped_pdf_factories::data_reader(&input).unwrap();
            assert_eq!(
                reader
                    .LoadAsync(expected.len() as u32)
                    .unwrap()
                    .join()
                    .unwrap() as usize,
                expected.len()
            );
            let mut actual = [0; 24];
            assert_eq!(actual.len(), expected.len());
            reader.ReadBytes(&mut actual).unwrap();
            assert_eq!(&actual, expected);

            let options = scoped_pdf_factories::render_options().unwrap();
            options.SetDestinationWidth(16).unwrap();
            options.SetDestinationHeight(8).unwrap();
            options.SetIsIgnoringHighContrast(true).unwrap();
            assert_eq!(options.DestinationWidth().unwrap(), 16);
            assert_eq!(options.DestinationHeight().unwrap(), 8);
            assert!(options.IsIgnoringHighContrast().unwrap());
            let png_id = scoped_pdf_factories::png_encoder_id().unwrap();
            assert_ne!(png_id, GUID::zeroed());
            options.SetBitmapEncoderId(png_id).unwrap();
            assert_eq!(options.BitmapEncoderId().unwrap(), png_id);
            *encoder_id = png_id;

            // All joined operations are already dropped. Explicitly release
            // every owned interface before this iteration's apartment guard.
            drop(options);
            drop(reader);
            drop(input);
            drop(writer);
            drop(stream);
        }
        assert_eq!(encoder_ids[0], encoder_ids[1]);
    }
    // P10 TWO-APARTMENT TEST END

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
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        let diagnostic_phase = pdf_test_diagnostic::PhaseGuard::valid();
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::case(pdf_test_diagnostic::Case::ValidBegin);
        // P9 TEST-ONLY END
        let original = synthetic_pdf(2);
        let processed = process_in_worker(InputKind::Pdf, &original).unwrap();
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::case(pdf_test_diagnostic::Case::ValidEnd);
        // P9 TEST-ONLY END
        assert_eq!(processed.source_sha256, source_hash(&original));
        let ProcessedContent::Pages(pages) = processed.content else {
            panic!("pages")
        };
        assert_eq!(pages.len(), 2);
        assert_eq!((pages[0].width, pages[0].height), (150, 150));
        let centre = (75 * 150 + 75) * 4;
        assert_eq!(&pages[0].rgba[centre..centre + 4], &[255, 0, 0, 255]);
        assert_eq!(&pages[1].rgba[centre..centre + 4], &[0, 0, 255, 255]);
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::case(pdf_test_diagnostic::Case::AssertionsDone);
        #[cfg(all(test, windows))]
        diagnostic_phase.limit();
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::case(pdf_test_diagnostic::Case::LimitBegin);
        // P9 TEST-ONLY END
        assert!(matches!(
            process_in_worker(InputKind::Pdf, &synthetic_pdf(MAX_PDF_PAGES + 1)),
            Err(ProcessingError::Limit)
        ));
        // P9 TEST-ONLY BEGIN
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::case(pdf_test_diagnostic::Case::LimitEnd);
        #[cfg(all(test, windows))]
        pdf_test_diagnostic::case(pdf_test_diagnostic::Case::Complete);
        // P9 TEST-ONLY END
    }
}
