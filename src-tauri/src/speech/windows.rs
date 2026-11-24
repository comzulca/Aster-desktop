use super::{Control, ACTIVE, CANCEL};
mod decoder;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    ffi::OsString,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    mem::size_of,
    os::windows::{
        ffi::OsStringExt,
        fs::{MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
    ptr::null_mut,
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::{Duration, Instant},
};
use windows_sys::Win32::{
    Media::Audio::{
        waveInAddBuffer, waveInClose, waveInOpen, waveInPrepareHeader, waveInReset, waveInStart,
        waveInUnprepareHeader, WAVEFORMATEX, WAVEHDR, WAVE_MAPPER,
    },
    Storage::FileSystem::{
        FILE_ATTRIBUTE_REPARSE_POINT, FILE_FLAG_BACKUP_SEMANTICS, FILE_FLAG_OPEN_REPARSE_POINT,
        FILE_SHARE_READ,
    },
    System::SystemInformation::GetSystemDirectoryW,
};
use zeroize::Zeroizing;

const UNAVAILABLE: &str = "speech_unavailable";
const FAILED: &str = "speech_failed";
const RELEASE_UNCONFIRMED: &str = "speech_release_unconfirmed";
const MODEL: &str = "ggml-base-q5_1.bin";
const SAMPLE_RATE: u32 = 16000;
const MAX_SECONDS: u32 = 30;
const POLL: Duration = Duration::from_millis(25);
const MAX_TEXT: usize = crate::policy::MAX_MESSAGE_BYTES;
static DEVICE_FAILURE: AtomicBool = AtomicBool::new(false);

#[derive(Deserialize)]
struct Asset {
    bytes: u64,
    sha256: String,
}

pub(super) struct VerifiedAssets {
    directory: PathBuf,
    // Deny replacement or writes while the CPU engine can open its libraries/model.
    _handles: Vec<File>,
}

impl VerifiedAssets {
    pub(super) fn open(directory: &Path) -> Result<Self, &'static str> {
        let manifest: BTreeMap<String, Asset> =
            serde_json::from_str(include_str!("../../speech-assets.json"))
                .map_err(|_| UNAVAILABLE)?;
        let directory_handle = OpenOptions::new()
            .read(true)
            .share_mode(FILE_SHARE_READ)
            .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT | FILE_FLAG_BACKUP_SEMANTICS)
            .open(directory)
            .map_err(|_| UNAVAILABLE)?;
        let meta = directory_handle.metadata().map_err(|_| UNAVAILABLE)?;
        if !meta.is_dir() || meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0 {
            return Err(UNAVAILABLE);
        }
        let actual = fs::read_dir(directory)
            .map_err(|_| UNAVAILABLE)?
            .map(|entry| entry.map(|entry| entry.file_name()))
            .collect::<Result<BTreeSet<_>, _>>()
            .map_err(|_| UNAVAILABLE)?;
        if actual != manifest.keys().map(OsString::from).collect() {
            return Err(UNAVAILABLE);
        }
        let mut handles = vec![directory_handle];
        for (name, asset) in manifest {
            let mut file = OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ)
                .custom_flags(FILE_FLAG_OPEN_REPARSE_POINT)
                .open(directory.join(name))
                .map_err(|_| UNAVAILABLE)?;
            let meta = file.metadata().map_err(|_| UNAVAILABLE)?;
            if !meta.is_file()
                || meta.len() != asset.bytes
                || meta.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
            {
                return Err(UNAVAILABLE);
            }
            let mut hash = Sha256::new();
            let mut buf = [0; 16384];
            loop {
                let n = file.read(&mut buf).map_err(|_| UNAVAILABLE)?;
                if n == 0 {
                    break;
                }
                hash.update(&buf[..n]);
            }
            if hash
                .finalize()
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
                != asset.sha256
            {
                return Err(UNAVAILABLE);
            }
            handles.push(file);
        }
        Ok(Self {
            directory: directory.to_owned(),
            _handles: handles,
        })
    }
}

fn capture_close_result(
    closed: bool,
    action: u8,
    device_failure: &AtomicBool,
) -> Result<(), &'static str> {
    if !closed {
        device_failure.store(true, Ordering::Release);
        return Err(RELEASE_UNCONFIRMED);
    }
    if action == CANCEL {
        return Err("speech_cancelled");
    }
    Ok(())
}

pub(super) fn capture(
    control: &Control,
    on_recording: impl FnOnce() -> Result<(), &'static str>,
) -> Result<Zeroizing<Vec<u8>>, &'static str> {
    if DEVICE_FAILURE.load(Ordering::Acquire) {
        return Err(RELEASE_UNCONFIRMED);
    }
    if control.action() != ACTIVE {
        return Err("speech_cancelled");
    }
    let format = WAVEFORMATEX {
        wFormatTag: 1,
        nChannels: 1,
        nSamplesPerSec: SAMPLE_RATE,
        nAvgBytesPerSec: SAMPLE_RATE * 2,
        nBlockAlign: 2,
        wBitsPerSample: 16,
        cbSize: 0,
    };
    let mut device = null_mut();
    let mut pcm = Zeroizing::new(vec![0u8; (SAMPLE_RATE * 2 * MAX_SECONDS) as usize]);
    let mut header = Box::new(WAVEHDR {
        lpData: pcm.as_mut_ptr(),
        dwBufferLength: pcm.len() as u32,
        ..Default::default()
    });
    let header_bytes = size_of::<WAVEHDR>() as u32;
    let header_ptr = &raw mut *header;
    // The header and PCM both retain fixed heap addresses until Windows synchronously
    // returns the buffer. No Rust reference to driver-written data is read mid-capture.
    unsafe {
        if waveInOpen(&mut device, WAVE_MAPPER, &format, 0, 0, 0) != 0 {
            return Err("speech_microphone");
        }
        if waveInPrepareHeader(device, header_ptr, header_bytes) != 0 {
            capture_close_result(waveInClose(device) == 0, control.action(), &DEVICE_FAILURE)?;
            return Err("speech_microphone");
        }
        let queued = waveInAddBuffer(device, header_ptr, header_bytes) == 0;
        let running = queued && control.action() == ACTIVE && waveInStart(device) == 0;
        // The event emitter is an external callback. An unwind must not free
        // buffers that Windows still owns; always reset/unprepare before exit.
        let capture_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            if running {
                let deadline = Instant::now() + Duration::from_secs(MAX_SECONDS.into());
                on_recording()?;
                while control.action() == ACTIVE && Instant::now() < deadline {
                    thread::sleep(POLL);
                }
            }
            Ok::<_, &'static str>(())
        }));
        // Reset synchronously returns queued and partial buffers. On a broken
        // driver leave any still-owned allocation alive rather than freeing it.
        let reset = waveInReset(device);
        let released = waveInUnprepareHeader(device, header_ptr, header_bytes) == 0;
        let closed = waveInClose(device) == 0;
        if let Err(reason) = capture_close_result(closed, control.action(), &DEVICE_FAILURE) {
            if !released && !closed {
                std::mem::forget(header);
                std::mem::forget(pcm);
            }
            return Err(reason);
        }
        capture_result.map_err(|_| FAILED)??;
        if !running || reset != 0 || !released {
            return Err("speech_microphone");
        }
    }
    let recorded = header.dwBytesRecorded as usize;
    if recorded > pcm.len() || !recorded.is_multiple_of(2) {
        return Err(FAILED);
    }
    pcm.truncate(recorded);
    if recorded < (SAMPLE_RATE / 2) as usize
        || !pcm
            .chunks_exact(2)
            .any(|s| i16::from_le_bytes([s[0], s[1]]).unsigned_abs() > 64)
    {
        return Err("speech_no_audio");
    }
    Ok(pcm)
}

// Only a fixed PCM16/16kHz/mono RIFF stream is passed through stdin.
fn write_wav(out: &mut impl Write, pcm: &[u8]) -> std::io::Result<()> {
    let n = pcm.len() as u32;
    out.write_all(b"RIFF")?;
    out.write_all(&(n + 36).to_le_bytes())?;
    out.write_all(b"WAVEfmt \x10\0\0\0\x01\0\x01\0")?;
    out.write_all(&SAMPLE_RATE.to_le_bytes())?;
    out.write_all(&(2 * SAMPLE_RATE).to_le_bytes())?;
    out.write_all(b"\x02\0\x10\0data")?;
    out.write_all(&n.to_le_bytes())?;
    out.write_all(pcm)
}

fn system_directory() -> Result<PathBuf, &'static str> {
    let mut buf = [0u16; 32768];
    let n = unsafe { GetSystemDirectoryW(buf.as_mut_ptr(), buf.len() as u32) } as usize;
    if n == 0 || n >= buf.len() {
        return Err(UNAVAILABLE);
    }
    Ok(PathBuf::from(OsString::from_wide(&buf[..n])))
}

pub(super) fn transcribe(
    assets: &VerifiedAssets,
    pcm: Zeroizing<Vec<u8>>,
    control: &Control,
) -> Result<String, &'static str> {
    if control.action() == CANCEL {
        return Err("speech_cancelled");
    }
    let (worker, mut input, output) = decoder::Decoder::start(assets)?;
    // Pipes are drained concurrently. The parent retains its deadline and can
    // always kill the child before joining either I/O worker.
    let writer = thread::Builder::new()
        .spawn(move || write_wav(&mut input, &pcm))
        .map_err(|_| FAILED)?;
    let reader = thread::Builder::new()
        .spawn(move || {
            let mut data = Vec::new();
            output
                .take(MAX_TEXT as u64 + 1)
                .read_to_end(&mut data)
                .map(|_| data)
        })
        .map_err(|_| FAILED)?;
    let deadline = Instant::now() + Duration::from_secs(120);
    let completion = loop {
        if control.action() == CANCEL {
            break Err("speech_cancelled");
        }
        if Instant::now() >= deadline {
            break Err("speech_timeout");
        }
        match worker.try_wait() {
            Ok(Some(0)) => break Ok(()),
            Ok(Some(0xc0000135)) => break Err("speech_runtime"),
            Ok(Some(_)) | Err(_) => break Err(FAILED),
            Ok(None) => thread::sleep(POLL),
        }
    };
    drop(worker);
    let wrote = writer.join();
    let read = reader.join();
    completion?;
    wrote.map_err(|_| FAILED)?.map_err(|_| FAILED)?;
    let data = read.map_err(|_| FAILED)?.map_err(|_| FAILED)?;
    if control.action() == CANCEL {
        return Err("speech_cancelled");
    }
    decode_output(data)
}

fn decode_output(data: Vec<u8>) -> Result<String, &'static str> {
    if data.len() > MAX_TEXT {
        return Err(FAILED);
    }
    let text = String::from_utf8(data).map_err(|_| FAILED)?;
    if text
        .chars()
        .any(|c| c.is_control() && !matches!(c, '\r' | '\n' | '\t'))
    {
        return Err(FAILED);
    }
    let draft = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if draft.is_empty() {
        Err("speech_no_audio")
    } else {
        Ok(draft)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cancellation_cannot_certify_a_failed_device_close() {
        let device_failure = AtomicBool::new(false);
        assert_eq!(
            capture_close_result(false, CANCEL, &device_failure),
            Err("speech_release_unconfirmed")
        );
        assert!(device_failure.load(Ordering::Acquire));
        let healthy = AtomicBool::new(false);
        assert_eq!(
            capture_close_result(true, CANCEL, &healthy),
            Err("speech_cancelled")
        );
        assert!(capture_close_result(true, ACTIVE, &healthy).is_ok());
        assert!(!healthy.load(Ordering::Acquire));
    }

    #[test]
    fn pipe_wave_has_exact_pcm_geometry_and_length() {
        let mut bytes = vec![];
        write_wav(&mut bytes, &[0, 1, 2, 3]).unwrap();
        assert_eq!(bytes.len(), 48);
        assert_eq!(&bytes[36..44], b"data\x04\x00\x00\x00");
        assert_eq!(&bytes[44..], &[0, 1, 2, 3]);
    }
    #[test]
    fn transcript_requires_bounded_valid_text() {
        assert_eq!(decode_output(b" uno\r\n dos ".to_vec()).unwrap(), "uno dos");
        for bytes in [vec![255], vec![b'a'; MAX_TEXT + 1], vec![0], vec![b' ']] {
            assert!(decode_output(bytes).is_err());
        }
    }

    #[test]
    fn production_decoder_transcribes_pinned_public_pcm_without_writing_files() {
        let reports = Path::new(env!("CARGO_MANIFEST_DIR")).join("../reports");
        let directory = reports.join("speech");
        assert!(
            VerifiedAssets::open(&reports).is_err(),
            "Extra/unlisted resources must be denied"
        );
        let assets = VerifiedAssets::open(&directory).expect("run the speech provisioner first");
        let wav = fs::read(reports.join("speech-fixture.wav")).unwrap();
        let digest = Sha256::digest(&wav)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>();
        assert_eq!(
            digest,
            "59dfb9a4acb36fe2a2affc14bacbee2920ff435cb13cc314a08c13f66ba7860e"
        );
        // This exact public JFK RIFF file has a LIST chunk before its PCM data.
        assert_eq!(&wav[70..78], b"data\x00\x5f\x05\x00");
        let control = Control {
            id: "ab".repeat(16),
            action: super::super::FINISH.into(),
            cancel_review: tokio::sync::watch::channel(false).0,
        };
        let result = transcribe(&assets, Zeroizing::new(wav[78..].to_vec()), &control).unwrap();
        assert!(result.to_lowercase().contains("country can do for you"));
        assert!(
            VerifiedAssets::open(&directory).is_ok(),
            "No transcript sidecar may be written"
        );
        control.signal(CANCEL);
        assert_eq!(
            transcribe(&assets, Zeroizing::new(vec![]), &control),
            Err("speech_cancelled")
        );
    }
}
