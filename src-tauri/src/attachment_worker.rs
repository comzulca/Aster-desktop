//! Fixed, byte-only child protocol and fail-closed Windows resource boundary.
//! A restricted low-integrity token/Job is NOT a no-network security sandbox.

use crate::attachment_processing::{
    self, InputKind, PdfFailureStage, ProcessedAttachment, ProcessedContent, ProcessingError,
    MAX_PDF_PAGES, MAX_PNG_BYTES, MAX_RGBA_BYTES, MAX_SOURCE_BYTES, MAX_TEXT_BYTES,
    MAX_WORKER_OUTPUT_BYTES,
};
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use zeroize::Zeroizing;

pub const WORKER_ARGUMENT: &str = "--aster-attachment-worker-v1";
pub const RESOURCE_WORKER_ARGUMENT: &str = "--aster-attachment-resource-worker-v1";
const RESOURCE_MAGIC: &[u8; 8] = b"ASTERJ01";
const INPUT_MAGIC: &[u8; 8] = b"ASTERI01";
const OUTPUT_MAGIC: &[u8; 8] = b"ASTERO01";
const MAX_ERROR_BYTES: usize = 64 * 1024;
pub const WORKER_MEMORY_BYTES: usize = 256 * 1024 * 1024;
pub const WORKER_CPU_SECONDS: u64 = 10;
pub const WORKER_WALL_SECONDS: u64 = 15;
#[cfg(any(windows, test))]
const WORKER_CPU_TICKS: i64 = (WORKER_CPU_SECONDS * 10_000_000) as i64;
#[cfg(any(windows, test))]
const CPU_WATCHDOG_EXIT: u32 = 0xA57E0001;

/// Parent-authored, bounded causal evidence; never supplied by the child.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize)]
pub enum CpuMechanism {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "native-job-cpu-limit")]
    NativeJobCpuLimit,
    #[serde(rename = "parent-user-cpu-watchdog")]
    ParentUserCpuWatchdog,
}

#[derive(Clone, Copy, Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CpuProof {
    pub mechanism: CpuMechanism,
    pub trigger_user_100ns: i64,
    pub trigger_wall_millis: u64,
    pub sampling_valid: bool,
    pub live_child_observed: bool,
    pub priority_rechecked: bool,
    pub exact_job_dispatched: bool,
    pub physical_expected_exit: bool,
    pub cleanup_complete: bool,
    pub terminal_denied: bool,
    pub exact_marker: bool,
    pub stats_valid: bool,
    pub natural_exit: bool,
    pub job_closed: bool,
    pub worker_error: bool,
    pub fallback_used: bool,
}

#[cfg(any(windows, test))]
impl CpuProof {
    fn passes(&self, boundary: bool, exit: Option<u32>, final_cpu: i64, native_count: u32) -> bool {
        if !boundary
            || !self.exact_marker
            || !self.stats_valid
            || !self.sampling_valid
            || !self.cleanup_complete
            || !self.terminal_denied
            || !self.worker_error
            || self.job_closed
            || self.fallback_used
            || final_cpu < WORKER_CPU_TICKS
        {
            return false;
        }
        match self.mechanism {
            CpuMechanism::NativeJobCpuLimit => {
                // Retain the original native branch, including its actual OS
                // termination counter. Explicit Job termination cannot forge it.
                self.natural_exit
                    && exit.is_some_and(|code| code != 0)
                    && native_count > 0
                    && self.trigger_user_100ns == 0
                    && self.trigger_wall_millis == 0
                    && !self.live_child_observed
                    && !self.priority_rechecked
                    && !self.exact_job_dispatched
                    && !self.physical_expected_exit
            }
            CpuMechanism::ParentUserCpuWatchdog => {
                !self.natural_exit
                    && self.live_child_observed
                    && self.priority_rechecked
                    && self.exact_job_dispatched
                    && self.physical_expected_exit
                    && self.trigger_user_100ns >= WORKER_CPU_TICKS
                    && self.trigger_wall_millis < WORKER_WALL_SECONDS * 1000
                    && final_cpu >= self.trigger_user_100ns
                    && exit == Some(CPU_WATCHDOG_EXIT)
            }
            CpuMechanism::None => false,
        }
    }
}

#[cfg(any(windows, test))]
fn user_cpu_ticks(low: u32, high: u32) -> Result<i64, ProcessingError> {
    let combined = (u64::from(high) << 32) | u64::from(low);
    i64::try_from(combined).map_err(|_| ProcessingError::Isolation)
}

#[cfg(any(windows, test))]
#[derive(Default)]
struct CpuSampling {
    previous: Option<i64>,
    failed: bool,
}

#[cfg(any(windows, test))]
impl CpuSampling {
    fn observe(&mut self, sample: Result<i64, ProcessingError>) -> Result<i64, ProcessingError> {
        match sample {
            Ok(value)
                if !self.failed
                    && value >= 0
                    && self.previous.is_none_or(|previous| value >= previous) =>
            {
                self.previous = Some(value);
                Ok(value)
            }
            _ => {
                self.failed = true;
                Err(ProcessingError::Isolation)
            }
        }
    }
    fn valid(&self) -> bool {
        !self.failed && self.previous.is_some()
    }
}

#[cfg(any(windows, test))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TerminalCause {
    Running,
    Cancel,
    Wall,
    Cpu,
    Other,
}

#[cfg(any(windows, test))]
impl TerminalCause {
    fn settle(&mut self, cause: Self) {
        if *self == Self::Running {
            *self = cause;
        }
    }
}

#[cfg(any(windows, test))]
fn cpu_decision(
    sample: Result<i64, ProcessingError>,
    cancelled: bool,
    wall_millis: u64,
) -> TerminalCause {
    if cancelled {
        TerminalCause::Cancel
    } else if wall_millis >= WORKER_WALL_SECONDS * 1000 {
        TerminalCause::Wall
    } else {
        match sample {
            Err(_) => TerminalCause::Other,
            Ok(ticks) if ticks < 0 => TerminalCause::Other,
            Ok(ticks) if ticks >= WORKER_CPU_TICKS => TerminalCause::Cpu,
            Ok(_) => TerminalCause::Running,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ResourceScenario {
    Cpu = 1,
    Memory = 2,
    Wall = 3,
    ActiveProcess = 4,
    KillOnClose = 5,
    Cancel = 6,
}
impl ResourceScenario {
    pub const ALL: [Self; 6] = [
        Self::Cpu,
        Self::Memory,
        Self::Wall,
        Self::ActiveProcess,
        Self::KillOnClose,
        Self::Cancel,
    ];
    pub fn code(self) -> &'static str {
        match self {
            Self::Cpu => "cpu-kill",
            Self::Memory => "memory-denial",
            Self::Wall => "wall-kill",
            Self::ActiveProcess => "active-process-denial",
            Self::KillOnClose => "kill-on-close",
            Self::Cancel => "cancel-kill",
        }
    }
    fn from_byte(byte: u8) -> Result<Self, ProcessingError> {
        Self::ALL
            .into_iter()
            .find(|scenario| *scenario as u8 == byte)
            .ok_or(ProcessingError::Invalid)
    }
}
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceProbeResult {
    pub code: &'static str,
    pub status: &'static str,
    pub reason: &'static str,
    pub elapsed_millis: u64,
    pub exit_code: Option<u32>,
    pub cpu_user_100ns: i64,
    pub peak_memory_bytes: usize,
    pub limit_terminations: u32,
    pub boundary_available: bool,
    pub cpu_proof: Option<CpuProof>,
}

/// Closed fixture-only operation, never exported as renderer IPC or a model tool.
pub fn run_resource_probe(scenario: ResourceScenario) -> ResourceProbeResult {
    #[cfg(windows)]
    {
        native::resource_probe(scenario)
    }
    #[cfg(not(windows))]
    {
        ResourceProbeResult {
            code: scenario.code(),
            status: "NOT RUN",
            reason: "windows-required",
            elapsed_millis: 0,
            exit_code: None,
            cpu_user_100ns: 0,
            peak_memory_bytes: 0,
            limit_terminations: 0,
            boundary_available: false,
            cpu_proof: None,
        }
    }
}

/// Requires the SAME effective token/Job before controlled synthetic work starts.
pub fn run_resource_worker_stdio() -> Result<(), ProcessingError> {
    #[cfg(windows)]
    {
        native::verify_child_boundary()?;
        let (kind, bytes) = read_request(&mut std::io::stdin().lock())?;
        if kind != InputKind::Text || bytes.len() != 1 {
            return Err(ProcessingError::Invalid);
        }
        let scenario = ResourceScenario::from_byte(bytes[0])?;
        let mut stdout = std::io::stdout().lock();
        stdout.write_all(RESOURCE_MAGIC).map_err(io_error)?;
        stdout.write_all(&[scenario as u8, 0]).map_err(io_error)?;
        stdout.flush().map_err(io_error)?;
        native::resource_child(scenario, &mut stdout)
    }
    #[cfg(not(windows))]
    Err(ProcessingError::Isolation)
}

/// Closed failure codes only: never serialize native error text or successful output.
pub(crate) fn worker_failure_exit(error: ProcessingError) -> u32 {
    match error {
        ProcessingError::Pdf(stage) => stage.worker_exit_code(),
        _ => 3,
    }
}

/// A code is meaningful only for the parent's exact PDF codec invocation.
/// Even a recognised stage remains terminal failure, never content admission.
fn worker_exit_error(kind: InputKind, codec_mode: bool, exit_code: u32) -> ProcessingError {
    if codec_mode && kind == InputKind::Pdf {
        if let Some(stage) = PdfFailureStage::from_worker_exit(exit_code) {
            return ProcessingError::Pdf(stage);
        }
    }
    ProcessingError::Worker
}

fn io_error(_: std::io::Error) -> ProcessingError {
    ProcessingError::Worker
}
fn read_u32(reader: &mut impl Read) -> Result<u32, ProcessingError> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes).map_err(io_error)?;
    Ok(u32::from_le_bytes(bytes))
}
fn write_u32(writer: &mut impl Write, n: usize) -> Result<(), ProcessingError> {
    let n = u32::try_from(n).map_err(|_| ProcessingError::Limit)?;
    writer.write_all(&n.to_le_bytes()).map_err(io_error)
}
fn require_eof(reader: &mut impl Read) -> Result<(), ProcessingError> {
    let mut extra = [0u8; 1];
    if reader.read(&mut extra).map_err(io_error)? != 0 {
        return Err(ProcessingError::Invalid);
    }
    Ok(())
}
fn write_request(
    writer: &mut impl Write,
    kind: InputKind,
    original: &[u8],
) -> Result<(), ProcessingError> {
    attachment_processing::validate_source(kind, original)?;
    writer.write_all(INPUT_MAGIC).map_err(io_error)?;
    writer.write_all(&[kind as u8]).map_err(io_error)?;
    write_u32(writer, original.len())?;
    writer.write_all(original).map_err(io_error)?;
    writer.flush().map_err(io_error)
}
fn read_request(
    reader: &mut impl Read,
) -> Result<(InputKind, Zeroizing<Vec<u8>>), ProcessingError> {
    let mut magic = [0u8; 8];
    reader.read_exact(&mut magic).map_err(io_error)?;
    if &magic != INPUT_MAGIC {
        return Err(ProcessingError::Invalid);
    }
    let mut kind = [0u8; 1];
    reader.read_exact(&mut kind).map_err(io_error)?;
    let kind = InputKind::try_from(kind[0])?;
    let length = read_u32(reader)? as usize;
    if length == 0
        || length > MAX_SOURCE_BYTES
        || (kind == InputKind::Text && length > MAX_TEXT_BYTES)
    {
        return Err(ProcessingError::Limit);
    }
    let mut original = Zeroizing::new(vec![0u8; length]);
    reader.read_exact(&mut original).map_err(io_error)?;
    require_eof(reader)?;
    attachment_processing::validate_source(kind, &original)?;
    Ok((kind, original))
}

fn encode_response(
    writer: &mut impl Write,
    kind: InputKind,
    original: &[u8],
    processed: &ProcessedAttachment,
) -> Result<(), ProcessingError> {
    writer.write_all(OUTPUT_MAGIC).map_err(io_error)?;
    writer.write_all(&[kind as u8]).map_err(io_error)?;
    writer
        .write_all(&Sha256::digest(original))
        .map_err(io_error)?;
    match &processed.content {
        ProcessedContent::Text(text) if kind == InputKind::Text => {
            attachment_processing::validate_source(kind, text.as_bytes())?;
            if text.as_bytes() != original {
                return Err(ProcessingError::Invalid);
            }
            writer.write_all(&[1]).map_err(io_error)?;
            write_u32(writer, text.len())?;
            writer.write_all(text.as_bytes()).map_err(io_error)?;
        }
        ProcessedContent::Pages(pages) if kind != InputKind::Text => {
            attachment_processing::validate_pages(pages)?;
            if kind != InputKind::Pdf && pages.len() != 1 {
                return Err(ProcessingError::Invalid);
            }
            writer.write_all(&[2]).map_err(io_error)?;
            write_u32(writer, pages.len())?;
            for page in pages {
                write_u32(writer, page.width as usize)?;
                write_u32(writer, page.height as usize)?;
                write_u32(writer, page.rgba.len())?;
                writer.write_all(&page.rgba).map_err(io_error)?;
                write_u32(writer, page.png.len())?;
                writer.write_all(&page.png).map_err(io_error)?;
            }
        }
        _ => return Err(ProcessingError::Invalid),
    }
    writer.flush().map_err(io_error)
}

/// Enter only before normal application bootstrap, with the sole fixed argument.
/// This entry receives bytes, not a filename, URL, credential or arbitrary command.
pub fn run_worker_stdio() -> Result<(), ProcessingError> {
    #[cfg(windows)]
    {
        native::verify_child_boundary()?;
        let (kind, original) = read_request(&mut std::io::stdin().lock())?;
        let processed = attachment_processing::process_in_worker(kind, &original)?;
        encode_response(&mut std::io::stdout().lock(), kind, &original, &processed)
    }
    #[cfg(not(windows))]
    Err(ProcessingError::Isolation)
}

/// Treat all child output as untrusted. Do not decode its encoded image in the
/// privileged parent: regenerate the canonical PNG from bounded RGBA and compare.
pub(crate) fn decode_response(
    kind: InputKind,
    original: &[u8],
    output: &[u8],
) -> Result<ProcessedAttachment, ProcessingError> {
    if output.len() > MAX_WORKER_OUTPUT_BYTES {
        return Err(ProcessingError::Limit);
    }
    let mut reader = std::io::Cursor::new(output);
    let mut magic = [0u8; 8];
    reader.read_exact(&mut magic).map_err(io_error)?;
    let mut declared_kind = [0u8; 1];
    reader.read_exact(&mut declared_kind).map_err(io_error)?;
    let mut source_digest = [0u8; 32];
    reader.read_exact(&mut source_digest).map_err(io_error)?;
    if &magic != OUTPUT_MAGIC
        || declared_kind[0] != kind as u8
        || source_digest.as_slice() != Sha256::digest(original).as_slice()
    {
        return Err(ProcessingError::Invalid);
    }
    let mut variant = [0u8; 1];
    reader.read_exact(&mut variant).map_err(io_error)?;
    let content = match (kind, variant[0]) {
        (InputKind::Text, 1) => {
            let length = read_u32(&mut reader)? as usize;
            if length == 0 || length > MAX_TEXT_BYTES || length != original.len() {
                return Err(ProcessingError::Limit);
            }
            let mut text = vec![0u8; length];
            reader.read_exact(&mut text).map_err(io_error)?;
            if text != original {
                return Err(ProcessingError::Invalid);
            }
            attachment_processing::validate_source(InputKind::Text, &text)?;
            ProcessedContent::Text(String::from_utf8(text).map_err(|_| ProcessingError::Invalid)?)
        }
        (InputKind::Png | InputKind::Jpeg | InputKind::Gif | InputKind::Pdf, 2) => {
            let count = read_u32(&mut reader)? as usize;
            if count == 0 || count > MAX_PDF_PAGES || (kind != InputKind::Pdf && count != 1) {
                return Err(ProcessingError::Limit);
            }
            let mut rgba_total = 0usize;
            let mut png_total = 0usize;
            let mut pages = Vec::with_capacity(count);
            for _ in 0..count {
                let width = read_u32(&mut reader)?;
                let height = read_u32(&mut reader)?;
                let rgba_length = read_u32(&mut reader)? as usize;
                if rgba_length != attachment_processing::validate_dimensions(width, height)? {
                    return Err(ProcessingError::Invalid);
                }
                rgba_total = rgba_total
                    .checked_add(rgba_length)
                    .ok_or(ProcessingError::Limit)?;
                if rgba_total > MAX_RGBA_BYTES {
                    return Err(ProcessingError::Limit);
                }
                let mut rgba = vec![0u8; rgba_length];
                reader.read_exact(&mut rgba).map_err(io_error)?;
                let png_length = read_u32(&mut reader)? as usize;
                png_total = png_total
                    .checked_add(png_length)
                    .ok_or(ProcessingError::Limit)?;
                if png_length == 0 || png_total > MAX_PNG_BYTES {
                    return Err(ProcessingError::Limit);
                }
                let mut claimed_png = vec![0u8; png_length];
                reader.read_exact(&mut claimed_png).map_err(io_error)?;
                let canonical = attachment_processing::page_from_rgba(width, height, rgba)?;
                if canonical.png != claimed_png {
                    return Err(ProcessingError::Invalid);
                }
                pages.push(canonical);
            }
            attachment_processing::validate_pages(&pages)?;
            ProcessedContent::Pages(pages)
        }
        _ => return Err(ProcessingError::Invalid),
    };
    require_eof(&mut reader)?;
    Ok(ProcessedAttachment {
        source_sha256: attachment_processing::source_hash(original),
        content,
    })
}

#[cfg(not(windows))]
pub(crate) fn run_isolated(
    _: InputKind,
    _: &[u8],
    _: &AtomicBool,
) -> Result<Vec<u8>, ProcessingError> {
    Err(ProcessingError::Isolation)
}

#[cfg(windows)]
pub(crate) fn run_isolated(
    kind: InputKind,
    original: &[u8],
    cancel: &AtomicBool,
) -> Result<Vec<u8>, ProcessingError> {
    native::run(kind, original, cancel)
}

#[cfg(windows)]
mod native {
    use super::*;
    use std::ffi::c_void;
    use std::fs::File;
    use std::mem::{size_of, size_of_val, zeroed};
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
    use std::ptr::{null, null_mut};
    use std::sync::{mpsc, Arc, Mutex};
    use std::thread::JoinHandle;
    use std::time::{Duration, Instant};
    use windows_sys::Win32::Foundation::{
        CloseHandle, GetLastError, SetHandleInformation, FILETIME, HANDLE, HANDLE_FLAG_INHERIT,
        INVALID_HANDLE_VALUE, WAIT_OBJECT_0, WAIT_TIMEOUT,
    };
    use windows_sys::Win32::Security::{
        AllocateAndInitializeSid, CreateRestrictedToken, FreeSid, GetLengthSid, GetSidSubAuthority,
        GetSidSubAuthorityCount, GetTokenInformation, IsValidSid, SetTokenInformation,
        TokenHasRestrictions, TokenIntegrityLevel, DISABLE_MAX_PRIVILEGE, PSID,
        SECURITY_ATTRIBUTES, SECURITY_MANDATORY_LABEL_AUTHORITY, SID_AND_ATTRIBUTES,
        SID_IDENTIFIER_AUTHORITY, TOKEN_ADJUST_DEFAULT, TOKEN_ASSIGN_PRIMARY, TOKEN_DUPLICATE,
        TOKEN_MANDATORY_LABEL, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::JobObjects::{
        AssignProcessToJobObject, CreateJobObjectW, IsProcessInJob,
        JobObjectAssociateCompletionPortInformation, JobObjectBasicAccountingInformation,
        JobObjectExtendedLimitInformation, QueryInformationJobObject, SetInformationJobObject,
        TerminateJobObject, JOBOBJECT_ASSOCIATE_COMPLETION_PORT,
        JOBOBJECT_BASIC_ACCOUNTING_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
        JOB_OBJECT_LIMIT_ACTIVE_PROCESS, JOB_OBJECT_LIMIT_BREAKAWAY_OK,
        JOB_OBJECT_LIMIT_JOB_MEMORY, JOB_OBJECT_LIMIT_JOB_TIME, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
        JOB_OBJECT_LIMIT_PROCESS_MEMORY, JOB_OBJECT_LIMIT_PROCESS_TIME,
        JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK,
    };
    use windows_sys::Win32::System::Memory::{
        VirtualAlloc, VirtualFree, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
    };
    use windows_sys::Win32::System::Pipes::CreatePipe;
    use windows_sys::Win32::System::SystemInformation::GetSystemDirectoryW;
    use windows_sys::Win32::System::SystemServices::{
        JOB_OBJECT_MSG_JOB_MEMORY_LIMIT, JOB_OBJECT_MSG_PROCESS_MEMORY_LIMIT,
    };
    use windows_sys::Win32::System::Threading::{
        CreateProcessAsUserW, CreateProcessW, DeleteProcThreadAttributeList, GetCurrentProcess,
        GetExitCodeProcess, GetProcessTimes, InitializeProcThreadAttributeList, OpenProcessToken,
        ResumeThread, TerminateProcess, UpdateProcThreadAttribute, WaitForSingleObject,
        CREATE_NO_WINDOW, CREATE_SUSPENDED, CREATE_UNICODE_ENVIRONMENT,
        EXTENDED_STARTUPINFO_PRESENT, LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION,
        PROC_THREAD_ATTRIBUTE_HANDLE_LIST, STARTF_USESTDHANDLES, STARTUPINFOEXW, STARTUPINFOW,
    };
    use windows_sys::Win32::System::IO::{
        CancelSynchronousIo, CreateIoCompletionPort, GetQueuedCompletionStatus,
    };

    // One executing allocation, or at most its three detached owned I/O threads
    // after failed cleanup. A poisoned boundary never admits another worker in
    // this process; restart is required. No arbitrary thread termination is used.
    static WORKER_ADMISSION: Mutex<()> = Mutex::new(());
    static WORKER_POISONED: AtomicBool = AtomicBool::new(false);
    const IO_CLEANUP_MILLIS: u64 = 1000;
    const IO_CLEANUP_MAX_ROUNDS: usize = 200;
    const MEMORY_NOTIFICATION_MILLIS: u64 = 250;
    const MEMORY_NOTIFICATION_MAX_PACKETS: usize = 64;

    struct Handle(HANDLE);
    impl Handle {
        fn new(raw: HANDLE) -> Result<Self, ProcessingError> {
            if raw.is_null() || raw as isize == -1 {
                Err(ProcessingError::Isolation)
            } else {
                Ok(Self(raw))
            }
        }
        fn into_file(self) -> File {
            let raw = self.0;
            std::mem::forget(self);
            // Unique ownership transfers to Rust File; no duplicate owner remains.
            unsafe { File::from(OwnedHandle::from_raw_handle(raw)) }
        }
    }
    impl Drop for Handle {
        fn drop(&mut self) {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
    struct Sid(PSID);
    impl Drop for Sid {
        fn drop(&mut self) {
            unsafe {
                FreeSid(self.0);
            }
        }
    }
    struct Attributes {
        _allocation: Vec<usize>,
        list: LPPROC_THREAD_ATTRIBUTE_LIST,
    }
    impl Drop for Attributes {
        fn drop(&mut self) {
            unsafe {
                DeleteProcThreadAttributeList(self.list);
            }
        }
    }
    struct JobKill {
        job: HANDLE,
        active: bool,
    }
    impl Drop for JobKill {
        fn drop(&mut self) {
            if self.active {
                unsafe {
                    TerminateJobObject(self.job, 1);
                }
            }
        }
    }

    #[derive(Clone, Copy)]
    enum Mode {
        Codec,
        Resource(ResourceScenario),
    }
    #[derive(Default)]
    struct JobStats {
        cpu_user: i64,
        peak_memory: usize,
        terminations: u32,
    }
    struct Execution {
        output: Vec<u8>,
        error: Option<ProcessingError>,
        elapsed_millis: u64,
        exit_code: Option<u32>,
        stats: JobStats,
        stats_valid: bool,
        ready: bool,
        natural_exit: bool,
        job_closed: bool,
        memory_limit_notification: bool,
        cpu_sampling_valid: bool,
        cpu_dispatch: Option<(i64, u64)>,
        cpu_live_observed: bool,
        cpu_priority_rechecked: bool,
        cpu_physical_exit: bool,
        cleanup_complete: bool,
        terminal_cause: TerminalCause,
        fallback_used: bool,
    }

    fn process_user_cpu(process: HANDLE) -> Result<i64, ProcessingError> {
        let mut creation: FILETIME = unsafe { zeroed() };
        let mut exit: FILETIME = unsafe { zeroed() };
        let mut kernel: FILETIME = unsafe { zeroed() };
        let mut user: FILETIME = unsafe { zeroed() };
        // This is the retained exact CreateProcess handle, not a PID lookup or
        // a child-supplied sample. Cumulative user CPU includes all child threads;
        // no startup baseline subtraction or kernel-time budget substitution.
        if unsafe { GetProcessTimes(process, &mut creation, &mut exit, &mut kernel, &mut user) }
            == 0
        {
            return Err(ProcessingError::Isolation);
        }
        user_cpu_ticks(user.dwLowDateTime, user.dwHighDateTime)
    }

    fn wall_millis(started: Instant) -> u64 {
        u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    fn memory_notification_matches(
        message: u32,
        key: usize,
        process_id: usize,
        expected_key: usize,
        expected_process_id: u32,
    ) -> bool {
        matches!(
            message,
            JOB_OBJECT_MSG_PROCESS_MEMORY_LIMIT | JOB_OBJECT_MSG_JOB_MEMORY_LIMIT
        ) && expected_key != 0
            && expected_process_id != 0
            && key == expected_key
            && process_id == expected_process_id as usize
    }

    struct JobNotifications {
        port: Handle,
        key: usize,
    }
    impl JobNotifications {
        fn associate_inactive(job: HANDLE) -> Result<Self, ProcessingError> {
            let port = Handle::new(unsafe {
                CreateIoCompletionPort(INVALID_HANDLE_VALUE, null_mut(), 0, 1)
            })?;
            no_inherit(&port)?;
            let association = JOBOBJECT_ASSOCIATE_COMPLETION_PORT {
                CompletionKey: job,
                CompletionPort: port.0,
            };
            if unsafe {
                SetInformationJobObject(
                    job,
                    JobObjectAssociateCompletionPortInformation,
                    &association as *const _ as *const c_void,
                    size_of::<JOBOBJECT_ASSOCIATE_COMPLETION_PORT>() as u32,
                )
            } == 0
            {
                return Err(ProcessingError::Isolation);
            }
            Ok(Self {
                port,
                key: job as usize,
            })
        }

        fn memory_limit_observed(&self, process_id: u32) -> Result<bool, ProcessingError> {
            let started = Instant::now();
            for _ in 0..MEMORY_NOTIFICATION_MAX_PACKETS {
                let remaining = Duration::from_millis(MEMORY_NOTIFICATION_MILLIS)
                    .saturating_sub(started.elapsed());
                if remaining.is_zero() {
                    break;
                }
                let mut message = 0u32;
                let mut key = 0usize;
                let mut value = null_mut();
                let dequeued = unsafe {
                    GetQueuedCompletionStatus(
                        self.port.0,
                        &mut message,
                        &mut key,
                        &mut value,
                        remaining.as_millis().min(20) as u32,
                    )
                } != 0;
                if !dequeued {
                    if unsafe { GetLastError() } == WAIT_TIMEOUT {
                        continue;
                    }
                    // No packet means the output parameters are indeterminate;
                    // failed packets are never accepted as Job limit evidence.
                    return Err(ProcessingError::Isolation);
                }
                // Job notifications put a PID, not an OVERLAPPED allocation, in
                // this value. Never dereference it. The child's process handle
                // remains open until this bounded drain ends, preventing reuse.
                if memory_notification_matches(message, key, value as usize, self.key, process_id) {
                    return Ok(true);
                }
            }
            // These Job messages are best-effort Windows notifications. Missing
            // evidence is unproved/FAIL, not proof the memory event never happened.
            Ok(false)
        }
    }

    fn job_stats(job: HANDLE) -> Result<JobStats, ProcessingError> {
        let mut accounting: JOBOBJECT_BASIC_ACCOUNTING_INFORMATION = unsafe { zeroed() };
        let mut memory: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
        if unsafe {
            QueryInformationJobObject(
                job,
                JobObjectBasicAccountingInformation,
                &mut accounting as *mut _ as *mut c_void,
                size_of::<JOBOBJECT_BASIC_ACCOUNTING_INFORMATION>() as u32,
                null_mut(),
            )
        } == 0
            || unsafe {
                QueryInformationJobObject(
                    job,
                    JobObjectExtendedLimitInformation,
                    &mut memory as *mut _ as *mut c_void,
                    size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                    null_mut(),
                )
            } == 0
        {
            return Err(ProcessingError::Isolation);
        }
        Ok(JobStats {
            cpu_user: accounting.TotalUserTime,
            peak_memory: memory.PeakProcessMemoryUsed,
            terminations: accounting.TotalTerminatedProcesses,
        })
    }

    pub(super) fn resource_child(
        scenario: ResourceScenario,
        output: &mut impl Write,
    ) -> Result<(), ProcessingError> {
        match scenario {
            ResourceScenario::Cpu => {
                let mut counter = 0u64;
                loop {
                    counter = counter.wrapping_add(1);
                    std::hint::black_box(counter);
                }
            }
            ResourceScenario::Wall | ResourceScenario::KillOnClose | ResourceScenario::Cancel => {
                loop {
                    std::thread::park();
                }
            }
            ResourceScenario::Memory => {
                // An explicit commit request above the hard cap, not an allocator
                // abort or disk write. A memory-limit denial need not kill a process.
                let allocation = unsafe {
                    VirtualAlloc(
                        null(),
                        WORKER_MEMORY_BYTES * 2,
                        MEM_COMMIT | MEM_RESERVE,
                        PAGE_READWRITE,
                    )
                };
                let denied = allocation.is_null();
                let error_code = if denied { unsafe { GetLastError() } } else { 0 };
                if !denied {
                    unsafe {
                        VirtualFree(allocation, 0, MEM_RELEASE);
                    }
                }
                output
                    .write_all(&[u8::from(denied && error_code != 0)])
                    .map_err(io_error)?;
                output
                    .write_all(&error_code.to_le_bytes())
                    .map_err(io_error)?;
                output.write_all(&0u32.to_le_bytes()).map_err(io_error)?;
            }
            ResourceScenario::ActiveProcess => {
                let before = job_stats(null_mut())?;
                let executable = std::env::current_exe().map_err(|_| ProcessingError::Isolation)?;
                let mut application: Vec<u16> = executable.as_os_str().encode_wide().collect();
                if application.is_empty()
                    || application.contains(&0)
                    || application.contains(&(b'"' as u16))
                {
                    return Err(ProcessingError::Isolation);
                }
                let mut command = vec![b'"' as u16];
                command.extend_from_slice(&application);
                command.extend("\" ".encode_utf16());
                command.extend(RESOURCE_WORKER_ARGUMENT.encode_utf16());
                command.push(0);
                application.push(0);
                let environment = [0u16; 2];
                let mut startup: STARTUPINFOW = unsafe { zeroed() };
                startup.cb = size_of::<STARTUPINFOW>() as u32;
                let mut process: PROCESS_INFORMATION = unsafe { zeroed() };
                let created = unsafe {
                    CreateProcessW(
                        application.as_ptr(),
                        command.as_mut_ptr(),
                        null(),
                        null(),
                        0,
                        CREATE_SUSPENDED | CREATE_NO_WINDOW | CREATE_UNICODE_ENVIRONMENT,
                        environment.as_ptr().cast(),
                        null(),
                        &startup,
                        &mut process,
                    )
                } != 0;
                let error_code = if created {
                    0
                } else {
                    unsafe { GetLastError() }
                };
                if created {
                    // Never resume an unexpectedly admitted child and never run UI.
                    let child = Handle::new(process.hProcess)?;
                    let _thread = Handle::new(process.hThread)?;
                    unsafe {
                        TerminateProcess(child.0, 1);
                        WaitForSingleObject(child.0, 1000);
                    }
                }
                let after = job_stats(null_mut())?;
                let terminations = after.terminations.saturating_sub(before.terminations);
                // ERROR_NOT_ENOUGH_QUOTA or an observed Job limit termination is
                // needed; an unrelated security/loader error is not a limit PASS.
                let denied_by_limit = !created && (error_code == 1816 || terminations > 0);
                output
                    .write_all(&[u8::from(denied_by_limit)])
                    .map_err(io_error)?;
                output
                    .write_all(&error_code.to_le_bytes())
                    .map_err(io_error)?;
                output
                    .write_all(&terminations.to_le_bytes())
                    .map_err(io_error)?;
            }
        }
        output.flush().map_err(io_error)
    }

    #[derive(Debug, PartialEq, Eq)]
    enum CleanupDecision {
        Finished,
        CancelAgain,
        FailClosed,
    }
    fn cleanup_decision(finished: bool, elapsed: Duration, rounds: usize) -> CleanupDecision {
        if finished {
            CleanupDecision::Finished
        } else if elapsed >= Duration::from_millis(IO_CLEANUP_MILLIS)
            || rounds >= IO_CLEANUP_MAX_ROUNDS
        {
            CleanupDecision::FailClosed
        } else {
            CleanupDecision::CancelAgain
        }
    }
    fn thread_terminated(thread: &JoinHandle<()>) -> bool {
        thread.is_finished()
            && unsafe { WaitForSingleObject(thread.as_raw_handle(), 0) } == WAIT_OBJECT_0
    }
    struct IoCancellation<'a> {
        stopped: Arc<AtomicBool>,
        threads: Vec<JoinHandle<()>>,
        job_to_terminate: Option<HANDLE>,
        process: Option<HANDLE>,
        poisoned: &'a AtomicBool,
        settled: bool,
    }
    impl IoCancellation<'_> {
        fn finish(&mut self) -> Result<(), ProcessingError> {
            self.stopped.store(true, Ordering::SeqCst);
            let started = Instant::now();
            let mut rounds = 0usize;
            loop {
                match cleanup_decision(
                    self.threads.iter().all(thread_terminated),
                    started.elapsed(),
                    rounds,
                ) {
                    CleanupDecision::Finished => {
                        self.settled = true;
                        let mut panicked = false;
                        // Both Rust completion and the Windows thread signal were
                        // observed. Never join an unfinished synchronous I/O call.
                        for thread in self.threads.drain(..) {
                            panicked |= thread.join().is_err();
                        }
                        return if panicked {
                            Err(ProcessingError::Worker)
                        } else {
                            Ok(())
                        };
                    }
                    CleanupDecision::FailClosed => {
                        self.poisoned.store(true, Ordering::SeqCst);
                        self.settled = true;
                        // JoinHandle::drop detaches; each closure still OWNS its
                        // File, cancellation Arc and bounded buffer. No borrowed
                        // raw thread handles survive. Zeroising the source copy
                        // occurs only when that thread eventually returns; cleanup
                        // is explicitly incomplete, not a secure-erasure claim.
                        self.threads.clear();
                        return Err(ProcessingError::Isolation);
                    }
                    CleanupDecision::CancelAgain => {
                        for thread in &self.threads {
                            if !thread_terminated(thread) {
                                // A cancellation before the next ReadFile or
                                // WriteFile sees ERROR_NOT_FOUND. Repeating while
                                // the owned thread is alive closes that race; a
                                // successful cancel alone is not completion proof.
                                unsafe {
                                    CancelSynchronousIo(thread.as_raw_handle());
                                }
                            }
                        }
                        rounds += 1;
                        std::thread::sleep(Duration::from_millis(5));
                    }
                }
            }
        }
    }
    impl Drop for IoCancellation<'_> {
        fn drop(&mut self) {
            if !self.settled {
                // Unwinding or a failed thread spawn must also kill before trying
                // to cancel the owned I/O threads. Outer Handle owners remain
                // alive throughout this guard's lifetime.
                if let Some(process) = self.process {
                    if let Some(job) = self.job_to_terminate {
                        if unsafe { TerminateJobObject(job, 1) } == 0 {
                            unsafe {
                                TerminateProcess(process, 1);
                            }
                        }
                    }
                    if unsafe { WaitForSingleObject(process, 1000) } != WAIT_OBJECT_0 {
                        self.poisoned.store(true, Ordering::SeqCst);
                    }
                }
                let _ = self.finish();
            }
        }
    }

    struct CancellableWriter {
        file: File,
        stopped: Arc<AtomicBool>,
    }
    impl Write for CancellableWriter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if self.stopped.load(Ordering::SeqCst) {
                return Err(std::io::Error::other("Cancelled"));
            }
            self.file.write(bytes)
        }
        fn flush(&mut self) -> std::io::Result<()> {
            if self.stopped.load(Ordering::SeqCst) {
                return Err(std::io::Error::other("Cancelled"));
            }
            self.file.flush()
        }
    }

    fn within_limits(limits: &JOBOBJECT_EXTENDED_LIMIT_INFORMATION) -> bool {
        let required = JOB_OBJECT_LIMIT_ACTIVE_PROCESS
            | JOB_OBJECT_LIMIT_PROCESS_MEMORY
            | JOB_OBJECT_LIMIT_JOB_MEMORY
            | JOB_OBJECT_LIMIT_PROCESS_TIME
            | JOB_OBJECT_LIMIT_JOB_TIME
            | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let flags = limits.BasicLimitInformation.LimitFlags;
        let cpu_limit = (WORKER_CPU_SECONDS * 10_000_000) as i64;
        flags & required == required
            && flags & (JOB_OBJECT_LIMIT_BREAKAWAY_OK | JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK) == 0
            && limits.BasicLimitInformation.ActiveProcessLimit == 1
            && (1..=WORKER_MEMORY_BYTES).contains(&limits.ProcessMemoryLimit)
            && (1..=WORKER_MEMORY_BYTES).contains(&limits.JobMemoryLimit)
            && (1..=cpu_limit).contains(&limits.BasicLimitInformation.PerProcessUserTimeLimit)
            && (1..=cpu_limit).contains(&limits.BasicLimitInformation.PerJobUserTimeLimit)
    }

    /// A fixed CLI argument is not a grant: inspect the actual child token and
    /// immediate Job before reading even the first input byte or calling a decoder.
    pub(super) fn verify_child_boundary() -> Result<(), ProcessingError> {
        let mut in_job = 0;
        if unsafe { IsProcessInJob(GetCurrentProcess(), null_mut(), &mut in_job) } == 0
            || in_job == 0
        {
            return Err(ProcessingError::Isolation);
        }
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
        if unsafe {
            QueryInformationJobObject(
                null_mut(),
                JobObjectExtendedLimitInformation,
                &mut limits as *mut _ as *mut c_void,
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                null_mut(),
            )
        } == 0
            || !within_limits(&limits)
        {
            return Err(ProcessingError::Isolation);
        }
        let mut token = null_mut();
        if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
            return Err(ProcessingError::Isolation);
        }
        let token = Handle::new(token)?;
        let mut filtered = 0u32;
        let mut length = 0u32;
        if unsafe {
            GetTokenInformation(
                token.0,
                TokenHasRestrictions,
                &mut filtered as *mut _ as *mut c_void,
                size_of::<u32>() as u32,
                &mut length,
            )
        } == 0
            || filtered == 0
        {
            return Err(ProcessingError::Isolation);
        }
        // Word-aligned storage, bounded independently of the operating-system result.
        let mut information = [0usize; 512];
        if unsafe {
            GetTokenInformation(
                token.0,
                TokenIntegrityLevel,
                information.as_mut_ptr().cast(),
                size_of_val(&information) as u32,
                &mut length,
            )
        } == 0
            || (length as usize) < size_of::<TOKEN_MANDATORY_LABEL>()
            || (length as usize) > size_of_val(&information)
        {
            return Err(ProcessingError::Isolation);
        }
        let label = unsafe { &*information.as_ptr().cast::<TOKEN_MANDATORY_LABEL>() };
        let sid = label.Label.Sid;
        let start = information.as_ptr() as usize;
        let end = start + length as usize;
        // TOKEN_MANDATORY_LABEL's SID points into its own returned buffer.
        if (sid as usize) < start || (sid as usize).checked_add(8).is_none_or(|n| n > end) {
            return Err(ProcessingError::Isolation);
        }
        let count = unsafe { *GetSidSubAuthorityCount(sid) };
        if count == 0
            || (sid as usize)
                .checked_add(8 + usize::from(count) * 4)
                .is_none_or(|n| n > end)
            || unsafe { IsValidSid(sid) } == 0
        {
            return Err(ProcessingError::Isolation);
        }
        if unsafe { *GetSidSubAuthority(sid, u32::from(count) - 1) } != 0x1000 {
            return Err(ProcessingError::Isolation);
        }
        Ok(())
    }

    fn pipe() -> Result<(Handle, Handle), ProcessingError> {
        let attributes = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: null_mut(),
            bInheritHandle: 1,
        };
        let (mut read, mut write) = (null_mut(), null_mut());
        if unsafe { CreatePipe(&mut read, &mut write, &attributes, 0) } == 0 {
            return Err(ProcessingError::Isolation);
        }
        Ok((Handle::new(read)?, Handle::new(write)?))
    }
    fn no_inherit(handle: &Handle) -> Result<(), ProcessingError> {
        if unsafe { SetHandleInformation(handle.0, HANDLE_FLAG_INHERIT, 0) } == 0 {
            return Err(ProcessingError::Isolation);
        }
        Ok(())
    }
    fn sid(authority: SID_IDENTIFIER_AUTHORITY, parts: &[u32]) -> Result<Sid, ProcessingError> {
        let mut values = [0u32; 8];
        if parts.is_empty() || parts.len() > values.len() {
            return Err(ProcessingError::Isolation);
        }
        values[..parts.len()].copy_from_slice(parts);
        let mut result = null_mut();
        if unsafe {
            AllocateAndInitializeSid(
                &authority,
                parts.len() as u8,
                values[0],
                values[1],
                values[2],
                values[3],
                values[4],
                values[5],
                values[6],
                values[7],
                &mut result,
            )
        } == 0
        {
            return Err(ProcessingError::Isolation);
        }
        Ok(Sid(result))
    }
    fn restricted_token() -> Result<Handle, ProcessingError> {
        let mut parent = null_mut();
        if unsafe {
            OpenProcessToken(
                GetCurrentProcess(),
                TOKEN_QUERY | TOKEN_DUPLICATE | TOKEN_ASSIGN_PRIMARY | TOKEN_ADJUST_DEFAULT,
                &mut parent,
            )
        } == 0
        {
            return Err(ProcessingError::Isolation);
        }
        let parent = Handle::new(parent)?;
        // Built-in Administrators is deny-only; privileges are disabled, never enabled.
        let administrators = sid(
            SID_IDENTIFIER_AUTHORITY {
                Value: [0, 0, 0, 0, 0, 5],
            },
            &[32, 544],
        )?;
        let deny = SID_AND_ATTRIBUTES {
            Sid: administrators.0,
            Attributes: 0,
        };
        let mut token = null_mut();
        if unsafe {
            CreateRestrictedToken(
                parent.0,
                DISABLE_MAX_PRIVILEGE,
                1,
                &deny,
                0,
                null(),
                0,
                null(),
                &mut token,
            )
        } == 0
        {
            return Err(ProcessingError::Isolation);
        }
        let token = Handle::new(token)?;
        // Windows SDK SECURITY_MANDATORY_LOW_RID = 0x1000 / SE_GROUP_INTEGRITY = 0x20.
        let low = sid(SECURITY_MANDATORY_LABEL_AUTHORITY, &[0x1000])?;
        let label = TOKEN_MANDATORY_LABEL {
            Label: SID_AND_ATTRIBUTES {
                Sid: low.0,
                Attributes: 0x20,
            },
        };
        if unsafe {
            SetTokenInformation(
                token.0,
                TokenIntegrityLevel,
                &label as *const _ as *const c_void,
                size_of::<TOKEN_MANDATORY_LABEL>() as u32 + GetLengthSid(low.0),
            )
        } == 0
        {
            return Err(ProcessingError::Isolation);
        }
        Ok(token)
    }
    fn job() -> Result<Handle, ProcessingError> {
        let job = Handle::new(unsafe { CreateJobObjectW(null(), null()) })?;
        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_ACTIVE_PROCESS
            | JOB_OBJECT_LIMIT_PROCESS_MEMORY
            | JOB_OBJECT_LIMIT_JOB_MEMORY
            | JOB_OBJECT_LIMIT_PROCESS_TIME
            | JOB_OBJECT_LIMIT_JOB_TIME
            | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        limits.BasicLimitInformation.ActiveProcessLimit = 1;
        // Job CPU counters are 100-nanosecond intervals, not wall time.
        limits.BasicLimitInformation.PerProcessUserTimeLimit =
            (WORKER_CPU_SECONDS * 10_000_000) as i64;
        limits.BasicLimitInformation.PerJobUserTimeLimit = (WORKER_CPU_SECONDS * 10_000_000) as i64;
        limits.ProcessMemoryLimit = WORKER_MEMORY_BYTES;
        limits.JobMemoryLimit = WORKER_MEMORY_BYTES;
        if unsafe {
            SetInformationJobObject(
                job.0,
                JobObjectExtendedLimitInformation,
                &limits as *const _ as *const c_void,
                size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
            )
        } == 0
        {
            return Err(ProcessingError::Isolation);
        }
        Ok(job)
    }
    fn inherited_attributes(handles: &[HANDLE; 3]) -> Result<Attributes, ProcessingError> {
        let mut length = 0usize;
        unsafe {
            InitializeProcThreadAttributeList(null_mut(), 1, 0, &mut length);
        }
        if length == 0 || length > 64 * 1024 {
            return Err(ProcessingError::Isolation);
        }
        let words = length.div_ceil(size_of::<usize>());
        let mut allocation = vec![0usize; words];
        let list = allocation.as_mut_ptr().cast();
        if unsafe { InitializeProcThreadAttributeList(list, 1, 0, &mut length) } == 0 {
            return Err(ProcessingError::Isolation);
        }
        let attributes = Attributes {
            _allocation: allocation,
            list,
        };
        if unsafe {
            UpdateProcThreadAttribute(
                attributes.list,
                0,
                PROC_THREAD_ATTRIBUTE_HANDLE_LIST as usize,
                handles.as_ptr().cast(),
                size_of::<[HANDLE; 3]>(),
                null_mut(),
                null(),
            )
        } == 0
        {
            return Err(ProcessingError::Isolation);
        }
        Ok(attributes)
    }
    fn bounded_read(
        mut reader: File,
        limit: usize,
        stopped: &AtomicBool,
        ready: &AtomicBool,
        prefix: Option<&[u8]>,
    ) -> Result<Vec<u8>, ProcessingError> {
        let mut bytes = Vec::new();
        let mut chunk = [0u8; 16 * 1024];
        loop {
            if stopped.load(Ordering::SeqCst) {
                return if prefix.is_some() {
                    Ok(bytes)
                } else {
                    Err(ProcessingError::Cancelled)
                };
            }
            let n = match reader.read(&mut chunk) {
                Ok(n) => n,
                Err(_) if stopped.load(Ordering::SeqCst) && prefix.is_some() => return Ok(bytes),
                Err(error) => return Err(io_error(error)),
            };
            if n == 0 {
                return Ok(bytes);
            }
            if bytes.len().checked_add(n).is_none_or(|v| v > limit) {
                return Err(ProcessingError::Limit);
            }
            bytes.extend_from_slice(&chunk[..n]);
            if prefix.is_some_and(|prefix| bytes.starts_with(prefix)) {
                ready.store(true, Ordering::SeqCst);
            }
        }
    }
    fn discard_errors(mut reader: File, stopped: &AtomicBool) -> Result<(), ProcessingError> {
        let mut chunk = [0u8; 4096];
        let mut total = 0usize;
        loop {
            if stopped.load(Ordering::SeqCst) {
                return Err(ProcessingError::Cancelled);
            }
            let n = reader.read(&mut chunk).map_err(io_error)?;
            if n == 0 {
                return Ok(());
            }
            total = total.checked_add(n).ok_or(ProcessingError::Limit)?;
            if total > MAX_ERROR_BYTES {
                return Err(ProcessingError::Limit);
            }
            // Never log or return child's potentially private error bytes.
            chunk[..n].fill(0);
        }
    }

    pub(super) fn run(
        kind: InputKind,
        original: &[u8],
        cancel: &AtomicBool,
    ) -> Result<Vec<u8>, ProcessingError> {
        let execution = execute(kind, original, cancel, Mode::Codec)?;
        match execution.error {
            Some(error) => Err(error),
            None => Ok(execution.output),
        }
    }

    fn execute(
        kind: InputKind,
        original: &[u8],
        cancel: &AtomicBool,
        mode: Mode,
    ) -> Result<Execution, ProcessingError> {
        if cancel.load(Ordering::SeqCst) {
            return Err(ProcessingError::Cancelled);
        }
        let _admission = WORKER_ADMISSION
            .try_lock()
            .map_err(|_| ProcessingError::Isolation)?;
        if WORKER_POISONED.load(Ordering::SeqCst) {
            return Err(ProcessingError::Isolation);
        }
        attachment_processing::validate_source(kind, original)?;
        let token = restricted_token()?;
        let mut job = Some(job()?);
        let job_handle = job.as_ref().ok_or(ProcessingError::Isolation)?.0;
        // Only the closed synthetic one-allocation memory probe needs this port:
        // an adversarial codec allocation loop must not build an undrained Job
        // notification queue. Associate it while its Job is inactive, before
        // AssignProcessToJobObject or ResumeThread. Only the three exact stdio
        // handles below enter the child's inheritance allow-list.
        let notifications = if matches!(mode, Mode::Resource(ResourceScenario::Memory)) {
            Some(JobNotifications::associate_inactive(job_handle)?)
        } else {
            None
        };
        let (child_input, parent_input) = pipe()?;
        let (parent_output, child_output) = pipe()?;
        let (parent_error, child_error) = pipe()?;
        for parent in [&parent_input, &parent_output, &parent_error] {
            no_inherit(parent)?;
        }
        let inherited = [child_input.0, child_output.0, child_error.0];
        let attributes = inherited_attributes(&inherited)?;
        let executable = std::env::current_exe().map_err(|_| ProcessingError::Isolation)?;
        let mut executable_w: Vec<u16> = executable.as_os_str().encode_wide().collect();
        if executable_w.is_empty()
            || executable_w.contains(&0)
            || executable_w.contains(&(b'"' as u16))
        {
            return Err(ProcessingError::Isolation);
        }
        // Explicit lpApplicationName prevents path-search/quoting substitution.
        let mut command = vec![b'"' as u16];
        command.extend_from_slice(&executable_w);
        command.extend("\" ".encode_utf16());
        command.extend(
            match mode {
                Mode::Codec => WORKER_ARGUMENT,
                Mode::Resource(_) => RESOURCE_WORKER_ARGUMENT,
            }
            .encode_utf16(),
        );
        command.push(0);
        executable_w.push(0);
        let mut system_directory = [0u16; 32768];
        let directory_length = unsafe {
            GetSystemDirectoryW(system_directory.as_mut_ptr(), system_directory.len() as u32)
        } as usize;
        if directory_length == 0 || directory_length >= system_directory.len() {
            return Err(ProcessingError::Isolation);
        }
        // An empty explicit environment inherits no API keys, proxy, DLL search,
        // user profile/account or arbitrary parent setting. CWD is the OS directory.
        let environment = [0u16; 2];
        let mut startup: STARTUPINFOEXW = unsafe { zeroed() };
        startup.StartupInfo.cb = size_of::<STARTUPINFOEXW>() as u32;
        startup.StartupInfo.dwFlags = STARTF_USESTDHANDLES;
        startup.StartupInfo.hStdInput = child_input.0;
        startup.StartupInfo.hStdOutput = child_output.0;
        startup.StartupInfo.hStdError = child_error.0;
        startup.lpAttributeList = attributes.list;
        let mut process: PROCESS_INFORMATION = unsafe { zeroed() };
        let started = Instant::now();
        if unsafe {
            CreateProcessAsUserW(
                token.0,
                executable_w.as_ptr(),
                command.as_mut_ptr(),
                null(),
                null(),
                1,
                CREATE_SUSPENDED
                    | CREATE_UNICODE_ENVIRONMENT
                    | EXTENDED_STARTUPINFO_PRESENT
                    | CREATE_NO_WINDOW,
                environment.as_ptr().cast(),
                system_directory.as_ptr(),
                &startup.StartupInfo,
                &mut process,
            )
        } == 0
        {
            // OS/security denial is terminal. No unrestricted launch fallback.
            return Err(ProcessingError::Isolation);
        }
        let process_handle = Handle::new(process.hProcess)?;
        let thread_handle = Handle::new(process.hThread)?;
        if unsafe { AssignProcessToJobObject(job_handle, process_handle.0) } == 0 {
            unsafe {
                TerminateProcess(process_handle.0, 1);
                // An unassigned suspended child is not covered by Job close.
                // Do not admit another worker if its exit cannot be observed.
                if WaitForSingleObject(process_handle.0, 1000) != WAIT_OBJECT_0 {
                    WORKER_POISONED.store(true, Ordering::SeqCst);
                }
            }
            return Err(ProcessingError::Isolation);
        }
        // Kill guard exists before any path can unwind or resume parser execution.
        let mut lifetime = JobKill {
            job: job_handle,
            active: true,
        };
        drop(child_input);
        drop(child_output);
        drop(child_error);
        let input = parent_input.into_file();
        let output = parent_output.into_file();
        let errors = parent_error.into_file();
        {
            let stopped = Arc::new(AtomicBool::new(false));
            let mut io_cancel = IoCancellation {
                stopped: stopped.clone(),
                threads: Vec::with_capacity(3),
                job_to_terminate: Some(job_handle),
                process: Some(process_handle.0),
                poisoned: &WORKER_POISONED,
                settled: false,
            };
            let (writer_tx, writer_rx) = mpsc::channel();
            // This one bounded copy exists only for the lifetime of the byte-write
            // thread and is zeroised on return/unwind; no borrowed source can leak.
            let original_copy = Zeroizing::new(original.to_vec());
            let writer_stopped = stopped.clone();
            let writer = std::thread::Builder::new()
                .spawn(move || {
                    let mut input = CancellableWriter {
                        file: input,
                        stopped: writer_stopped,
                    };
                    let result = write_request(&mut input, kind, &original_copy);
                    drop(input);
                    let _ = writer_tx.send(result);
                })
                .map_err(|_| ProcessingError::Isolation)?;
            io_cancel.threads.push(writer);
            let (reader_tx, reader_rx) = mpsc::channel();
            let reader_stopped = stopped.clone();
            let ready = Arc::new(AtomicBool::new(false));
            let reader_ready = ready.clone();
            let prefix = match mode {
                Mode::Codec => None,
                Mode::Resource(scenario) => {
                    let mut prefix = RESOURCE_MAGIC.to_vec();
                    prefix.extend([scenario as u8, 0]);
                    Some(prefix)
                }
            };
            let output_limit = match mode {
                Mode::Codec => MAX_WORKER_OUTPUT_BYTES,
                Mode::Resource(_) => 128,
            };
            let reader = std::thread::Builder::new()
                .spawn(move || {
                    let _ = reader_tx.send(bounded_read(
                        output,
                        output_limit,
                        &reader_stopped,
                        &reader_ready,
                        prefix.as_deref(),
                    ));
                })
                .map_err(|_| ProcessingError::Isolation)?;
            io_cancel.threads.push(reader);
            let (error_tx, error_rx) = mpsc::channel();
            let error_stopped = stopped.clone();
            let error_reader = std::thread::Builder::new()
                .spawn(move || {
                    let _ = error_tx.send(discard_errors(errors, &error_stopped));
                })
                .map_err(|_| ProcessingError::Isolation)?;
            io_cancel.threads.push(error_reader);
            let mut result = None;
            let mut terminal_cause = TerminalCause::Running;
            let mut cpu_sampling = CpuSampling::default();
            let initial_sample = cpu_sampling.observe(process_user_cpu(process_handle.0));
            match cpu_decision(
                initial_sample,
                cancel.load(Ordering::SeqCst),
                wall_millis(started),
            ) {
                TerminalCause::Running => {
                    if unsafe { ResumeThread(thread_handle.0) } != 1 {
                        terminal_cause.settle(TerminalCause::Other);
                        result = Some(Err(ProcessingError::Isolation));
                    }
                }
                cause => {
                    terminal_cause.settle(cause);
                    result = Some(Err(match cause {
                        TerminalCause::Cancel => ProcessingError::Cancelled,
                        TerminalCause::Wall => ProcessingError::Timeout,
                        _ => ProcessingError::Isolation,
                    }));
                }
            }
            let mut cpu_dispatch = None;
            let mut cpu_live_observed = false;
            let mut cpu_priority_rechecked = false;
            let mut cpu_physical_exit = false;
            let mut fallback_used = false;
            let mut writing = None;
            let mut reading = None;
            let mut stderr = None;
            let mut natural_exit = false;
            let mut job_closed = false;
            let mut stats_before_close = None;
            while result.is_none() {
                if cancel.load(Ordering::SeqCst) {
                    terminal_cause.settle(TerminalCause::Cancel);
                    result = Some(Err(ProcessingError::Cancelled));
                    break;
                }
                if started.elapsed() >= Duration::from_secs(WORKER_WALL_SECONDS) {
                    terminal_cause.settle(TerminalCause::Wall);
                    result = Some(Err(ProcessingError::Timeout));
                    break;
                }
                let sample = cpu_sampling.observe(process_user_cpu(process_handle.0));
                let decision =
                    cpu_decision(sample, cancel.load(Ordering::SeqCst), wall_millis(started));
                if decision != TerminalCause::Running {
                    match decision {
                        TerminalCause::Cpu => {
                            let Ok(trigger_cpu) = sample else {
                                terminal_cause.settle(TerminalCause::Other);
                                result = Some(Err(ProcessingError::Isolation));
                                break;
                            };
                            // A child already signalled cannot become evidence
                            // of a parent kill. The unchanged strict native branch
                            // can still qualify actual OS-native CPU termination.
                            match unsafe { WaitForSingleObject(process_handle.0, 0) } {
                                WAIT_TIMEOUT => {
                                    let trigger_wall = wall_millis(started);
                                    let priority = cpu_decision(
                                        sample,
                                        cancel.load(Ordering::SeqCst),
                                        trigger_wall,
                                    );
                                    if priority != TerminalCause::Cpu {
                                        terminal_cause.settle(priority);
                                        result = Some(Err(if priority == TerminalCause::Cancel {
                                            ProcessingError::Cancelled
                                        } else {
                                            ProcessingError::Timeout
                                        }));
                                        break;
                                    }
                                    // Final retained-handle live/priority check;
                                    // no untrusted work occurs between it and the
                                    // single fixed exact-Job dispatch below.
                                    if unsafe { WaitForSingleObject(process_handle.0, 0) }
                                        != WAIT_TIMEOUT
                                    {
                                        terminal_cause.settle(TerminalCause::Other);
                                        result = Some(Err(ProcessingError::Worker));
                                        break;
                                    }
                                    let trigger_wall = wall_millis(started);
                                    let priority = cpu_decision(
                                        sample,
                                        cancel.load(Ordering::SeqCst),
                                        trigger_wall,
                                    );
                                    if priority != TerminalCause::Cpu {
                                        terminal_cause.settle(priority);
                                        result = Some(Err(if priority == TerminalCause::Cancel {
                                            ProcessingError::Cancelled
                                        } else {
                                            ProcessingError::Timeout
                                        }));
                                        break;
                                    }
                                    terminal_cause.settle(TerminalCause::Cpu);
                                    cpu_live_observed = true;
                                    cpu_priority_rechecked = true;
                                    if !job_closed
                                        && unsafe {
                                            TerminateJobObject(job_handle, CPU_WATCHDOG_EXIT)
                                        } != 0
                                    {
                                        cpu_dispatch = Some((trigger_cpu, trigger_wall));
                                        result = Some(Err(ProcessingError::Worker));
                                    } else {
                                        fallback_used = true;
                                        result = Some(Err(ProcessingError::Isolation));
                                    }
                                    break;
                                }
                                WAIT_OBJECT_0 => {
                                    natural_exit = !job_closed;
                                    terminal_cause.settle(TerminalCause::Other);
                                    result = Some(Err(ProcessingError::Worker));
                                    break;
                                }
                                _ => {
                                    terminal_cause.settle(TerminalCause::Other);
                                    result = Some(Err(ProcessingError::Isolation));
                                    break;
                                }
                            }
                        }
                        cause => {
                            terminal_cause.settle(cause);
                            result = Some(Err(match cause {
                                TerminalCause::Cancel => ProcessingError::Cancelled,
                                TerminalCause::Wall => ProcessingError::Timeout,
                                _ => ProcessingError::Isolation,
                            }));
                            break;
                        }
                    }
                }
                if ready.load(Ordering::SeqCst) {
                    if matches!(mode, Mode::Resource(ResourceScenario::Cancel)) {
                        cancel.store(true, Ordering::SeqCst);
                    }
                    if matches!(mode, Mode::Resource(ResourceScenario::KillOnClose)) && !job_closed
                    {
                        stats_before_close = job_stats(job_handle).ok();
                        if stats_before_close.is_none() {
                            result = Some(Err(ProcessingError::Isolation));
                            break;
                        }
                        lifetime.active = false;
                        io_cancel.job_to_terminate = None;
                        drop(job.take());
                        job_closed = true;
                    }
                }
                if writing.is_none() {
                    writing = writer_rx.try_recv().ok();
                }
                if reading.is_none() {
                    reading = reader_rx.try_recv().ok();
                }
                if stderr.is_none() {
                    stderr = error_rx.try_recv().ok();
                }
                if let Some(Err(e)) = writing.as_ref() {
                    terminal_cause.settle(TerminalCause::Other);
                    result = Some(Err(*e));
                    break;
                }
                if let Some(Err(e)) = reading.as_ref() {
                    terminal_cause.settle(TerminalCause::Other);
                    result = Some(Err(*e));
                    break;
                }
                if let Some(Err(e)) = stderr.as_ref() {
                    terminal_cause.settle(TerminalCause::Other);
                    result = Some(Err(*e));
                    break;
                }
                match unsafe { WaitForSingleObject(process_handle.0, 20) } {
                    WAIT_TIMEOUT => (),
                    WAIT_OBJECT_0 => {
                        natural_exit = !job_closed;
                        let mut exit_code = 1;
                        if unsafe { GetExitCodeProcess(process_handle.0, &mut exit_code) } == 0 {
                            terminal_cause.settle(TerminalCause::Other);
                            result = Some(Err(ProcessingError::Worker));
                            break;
                        }
                        if exit_code != 0 {
                            terminal_cause.settle(TerminalCause::Other);
                            result = Some(Err(worker_exit_error(
                                kind,
                                matches!(mode, Mode::Codec),
                                exit_code,
                            )));
                            break;
                        }
                        if writing.is_some() && reading.is_some() && stderr.is_some() {
                            // Exit zero and valid bytes are insufficient if the
                            // exact child used the full CPU budget. Query again
                            // on its retained handle before content admission.
                            let sample = cpu_sampling.observe(process_user_cpu(process_handle.0));
                            let decision = cpu_decision(
                                sample,
                                cancel.load(Ordering::SeqCst),
                                wall_millis(started),
                            );
                            terminal_cause.settle(if decision == TerminalCause::Running {
                                TerminalCause::Other
                            } else {
                                decision
                            });
                            result = match decision {
                                TerminalCause::Running => reading.take(),
                                TerminalCause::Cancel => Some(Err(ProcessingError::Cancelled)),
                                TerminalCause::Wall => Some(Err(ProcessingError::Timeout)),
                                TerminalCause::Cpu => Some(Err(ProcessingError::Worker)),
                                _ => Some(Err(ProcessingError::Isolation)),
                            };
                        } else {
                            std::thread::yield_now();
                        }
                    }
                    _ => {
                        terminal_cause.settle(TerminalCause::Other);
                        result = Some(Err(ProcessingError::Worker));
                    }
                }
            }
            // Hard kill before pipe cleanup. Repeated I/O cancellation below is
            // bounded by elapsed time and rounds, not an OS scheduler guarantee.
            // An uncompleted cleanup poisons future admission rather than hanging
            // the caller or accumulating another set of owned I/O threads.
            unsafe {
                // A successful CPU dispatch is already the hard kill. A second
                // generic kill must not overwrite its distinct causal exit.
                if !job_closed && cpu_dispatch.is_none() && TerminateJobObject(job_handle, 1) == 0 {
                    fallback_used = true;
                    result = Some(Err(ProcessingError::Isolation));
                    TerminateProcess(process_handle.0, 1);
                }
                if WaitForSingleObject(process_handle.0, 1000) != WAIT_OBJECT_0 {
                    fallback_used = true;
                    result = Some(Err(ProcessingError::Isolation));
                    // A failed kill-on-close observation must not leave a parked
                    // fixture orphan. Teardown is not a passing mechanism result.
                    TerminateProcess(process_handle.0, 1);
                    if WaitForSingleObject(process_handle.0, 1000) != WAIT_OBJECT_0 {
                        WORKER_POISONED.store(true, Ordering::SeqCst);
                    }
                }
            }
            if cpu_dispatch.is_some() && !fallback_used {
                let mut cpu_exit = 0u32;
                cpu_physical_exit = unsafe { WaitForSingleObject(process_handle.0, 0) }
                    == WAIT_OBJECT_0
                    && unsafe { GetExitCodeProcess(process_handle.0, &mut cpu_exit) } != 0
                    && cpu_exit == CPU_WATCHDOG_EXIT;
                if cpu_physical_exit {
                    // Physical CPU exit is confirmed, not merely requested.
                    // These guards still own cleanup, but no longer redispatch
                    // generic termination during a subsequent I/O failure.
                    lifetime.active = false;
                    io_cancel.job_to_terminate = None;
                }
            }
            io_cancel.finish()?;
            let memory_limit_notification = match &notifications {
                Some(notifications) => notifications.memory_limit_observed(process.dwProcessId)?,
                None => false,
            };
            let mut exit_code = 0u32;
            let observed_exit = if unsafe { WaitForSingleObject(process_handle.0, 0) }
                == WAIT_OBJECT_0
                && unsafe { GetExitCodeProcess(process_handle.0, &mut exit_code) } != 0
            {
                Some(exit_code)
            } else {
                None
            };
            let stats = if job_closed {
                stats_before_close
            } else {
                job_stats(job_handle).ok()
            };
            let stats_valid = stats.is_some();
            let (output, error) = match result.unwrap_or(Err(ProcessingError::Worker)) {
                Ok(output) => (output, None),
                Err(error) => (
                    reading
                        .or_else(|| reader_rx.try_recv().ok())
                        .and_then(Result::ok)
                        .unwrap_or_default(),
                    Some(error),
                ),
            };
            Ok(Execution {
                output,
                error,
                elapsed_millis: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
                exit_code: observed_exit,
                stats: stats.unwrap_or_default(),
                stats_valid,
                ready: ready.load(Ordering::SeqCst),
                natural_exit,
                job_closed,
                memory_limit_notification,
                cpu_sampling_valid: cpu_sampling.valid(),
                cpu_dispatch,
                cpu_live_observed,
                cpu_priority_rechecked,
                cpu_physical_exit,
                cleanup_complete: io_cancel.settled && io_cancel.threads.is_empty(),
                terminal_cause,
                fallback_used,
            })
        }
    }

    pub(super) fn resource_probe(scenario: ResourceScenario) -> ResourceProbeResult {
        let cancel = AtomicBool::new(false);
        let execution = match execute(
            InputKind::Text,
            &[scenario as u8],
            &cancel,
            Mode::Resource(scenario),
        ) {
            Ok(execution) => execution,
            Err(_) => {
                return ResourceProbeResult {
                    code: scenario.code(),
                    status: "FAIL",
                    reason: "resource-boundary-unavailable",
                    elapsed_millis: 0,
                    exit_code: None,
                    cpu_user_100ns: 0,
                    peak_memory_bytes: 0,
                    limit_terminations: 0,
                    boundary_available: false,
                    cpu_proof: None,
                }
            }
        };
        let mut expected_prefix = RESOURCE_MAGIC.to_vec();
        expected_prefix.extend([scenario as u8, 0]);
        let marker = execution.output.starts_with(&expected_prefix);
        let boundary_available =
            execution.ready && execution.stats_valid && execution.exit_code.is_some();
        let finite_packet = marker && execution.output.len() == 19 && execution.output[10] == 1;
        let packet_terminations = if execution.output.len() == 19 {
            u32::from_le_bytes(execution.output[15..19].try_into().unwrap_or([0; 4]))
        } else {
            0
        };
        let no_extra_output = marker && execution.output.len() == 10;
        let cpu_proof = if scenario == ResourceScenario::Cpu {
            let mut proof = CpuProof {
                mechanism: CpuMechanism::None,
                trigger_user_100ns: 0,
                trigger_wall_millis: 0,
                sampling_valid: execution.cpu_sampling_valid,
                live_child_observed: execution.cpu_live_observed,
                priority_rechecked: execution.cpu_priority_rechecked,
                exact_job_dispatched: execution.cpu_dispatch.is_some(),
                physical_expected_exit: execution.cpu_physical_exit,
                cleanup_complete: execution.cleanup_complete,
                terminal_denied: execution.error.is_some(),
                exact_marker: no_extra_output,
                stats_valid: execution.stats_valid,
                natural_exit: execution.natural_exit,
                job_closed: execution.job_closed,
                worker_error: execution.error == Some(ProcessingError::Worker),
                fallback_used: execution.fallback_used,
            };
            if let Some((ticks, wall)) = execution.cpu_dispatch {
                if execution.terminal_cause == TerminalCause::Cpu {
                    proof.mechanism = CpuMechanism::ParentUserCpuWatchdog;
                    proof.trigger_user_100ns = ticks;
                    proof.trigger_wall_millis = wall;
                }
            } else if no_extra_output
                && execution.natural_exit
                && execution.error == Some(ProcessingError::Worker)
                && execution.exit_code.is_some_and(|code| code != 0)
                && execution.stats.cpu_user >= WORKER_CPU_TICKS
                && execution.stats.terminations > 0
            {
                proof.mechanism = CpuMechanism::NativeJobCpuLimit;
            }
            Some(proof)
        } else {
            None
        };
        let passed = boundary_available
            && match scenario {
                ResourceScenario::Cpu => cpu_proof.as_ref().is_some_and(|proof| {
                    proof.passes(
                        boundary_available,
                        execution.exit_code,
                        execution.stats.cpu_user,
                        execution.stats.terminations,
                    )
                }),
                ResourceScenario::Memory => {
                    finite_packet
                        && execution.memory_limit_notification
                        && execution.natural_exit
                        && execution.error.is_none()
                        && execution.exit_code == Some(0)
                        && execution.stats.peak_memory > 0
                        && execution.stats.peak_memory <= WORKER_MEMORY_BYTES
                }
                ResourceScenario::ActiveProcess => {
                    finite_packet
                        && execution.natural_exit
                        && execution.error.is_none()
                        && execution.exit_code == Some(0)
                        && (packet_terminations > 0
                            || u32::from_le_bytes(
                                execution.output[11..15].try_into().unwrap_or([0; 4]),
                            ) == 1816)
                }
                ResourceScenario::Wall => {
                    no_extra_output
                        && !execution.natural_exit
                        && execution.error == Some(ProcessingError::Timeout)
                        && execution.elapsed_millis >= WORKER_WALL_SECONDS * 1000
                }
                ResourceScenario::KillOnClose => {
                    no_extra_output
                        && execution.job_closed
                        && !execution.natural_exit
                        && matches!(execution.error, None | Some(ProcessingError::Worker))
                        && execution.elapsed_millis < WORKER_WALL_SECONDS * 1000
                }
                ResourceScenario::Cancel => {
                    no_extra_output
                        && !execution.natural_exit
                        && execution.error == Some(ProcessingError::Cancelled)
                }
            };
        ResourceProbeResult {
            code: scenario.code(),
            status: if passed { "PASS" } else { "FAIL" },
            reason: if passed {
                "observed-synthetic-resource-containment"
            } else if !boundary_available {
                "resource-boundary-unavailable"
            } else if scenario == ResourceScenario::Memory && !execution.memory_limit_notification {
                "job-memory-notification-unobserved"
            } else {
                "resource-outcome-mismatch"
            },
            elapsed_millis: execution.elapsed_millis,
            exit_code: execution.exit_code,
            cpu_user_100ns: execution.stats.cpu_user,
            peak_memory_bytes: execution.stats.peak_memory,
            limit_terminations: execution.stats.terminations,
            boundary_available,
            cpu_proof,
        }
    }

    #[cfg(test)]
    mod boundary_tests {
        use super::*;

        #[test]
        fn memory_notification_requires_exact_job_message_key_and_child_pid() {
            for message in [
                JOB_OBJECT_MSG_PROCESS_MEMORY_LIMIT,
                JOB_OBJECT_MSG_JOB_MEMORY_LIMIT,
            ] {
                assert!(memory_notification_matches(message, 0x1234, 73, 0x1234, 73));
                assert!(!memory_notification_matches(
                    message, 0x1235, 73, 0x1234, 73
                ));
                assert!(!memory_notification_matches(
                    message, 0x1234, 74, 0x1234, 73
                ));
                assert!(!memory_notification_matches(message, 0x1234, 0, 0x1234, 73));
                assert!(!memory_notification_matches(message, 0, 73, 0, 73));
                assert!(!memory_notification_matches(message, 0x1234, 0, 0x1234, 0));
            }
            // An unrelated VirtualAlloc error, including quota error 1816, is
            // neither a memory-limit Job message nor sufficient containment proof.
            for unrelated in [0, 1, 8, 11, 1455, 1816] {
                assert!(!memory_notification_matches(
                    unrelated, 0x1234, 73, 0x1234, 73
                ));
            }
        }

        #[test]
        fn cleanup_repeats_when_cancel_precedes_the_held_pipe_call() {
            // Deterministic model of the original check-before-syscall race:
            // the first cancellation observes no pending call, then the worker
            // enters a held-endpoint read, then a later cancellation completes it.
            // This is policy/control-flow proof, not an executed Windows pipe test.
            let mut pending_call = false;
            let mut finished = false;
            let mut cancellations = 0usize;
            for round in 0..3 {
                let decision =
                    cleanup_decision(finished, Duration::from_millis(round as u64 * 5), round);
                if decision == CleanupDecision::Finished {
                    break;
                }
                assert_eq!(decision, CleanupDecision::CancelAgain);
                cancellations += 1;
                if pending_call {
                    finished = true;
                } else {
                    // The thread had passed its stop check but not entered I/O
                    // when the first cancellation was issued: ERROR_NOT_FOUND.
                    pending_call = true;
                }
            }
            assert_eq!(cancellations, 2);
            assert!(finished);
            assert_eq!(
                cleanup_decision(finished, Duration::from_millis(10), 2),
                CleanupDecision::Finished
            );
        }

        #[test]
        fn cleanup_never_joins_a_live_thread_and_exhaustion_denies() {
            assert_eq!(
                cleanup_decision(false, Duration::ZERO, 0),
                CleanupDecision::CancelAgain
            );
            assert_eq!(
                cleanup_decision(
                    false,
                    Duration::from_millis(IO_CLEANUP_MILLIS - 1),
                    IO_CLEANUP_MAX_ROUNDS - 1
                ),
                CleanupDecision::CancelAgain
            );
            assert_eq!(
                cleanup_decision(false, Duration::from_millis(IO_CLEANUP_MILLIS), 0),
                CleanupDecision::FailClosed
            );
            assert_eq!(
                cleanup_decision(false, Duration::ZERO, IO_CLEANUP_MAX_ROUNDS),
                CleanupDecision::FailClosed
            );
            assert_eq!(
                cleanup_decision(true, Duration::from_secs(2), IO_CLEANUP_MAX_ROUNDS),
                CleanupDecision::Finished
            );
        }

        #[test]
        fn held_anonymous_pipe_read_cancels_after_the_stop_check_race() {
            use windows_sys::Win32::Foundation::{ERROR_NOT_FOUND, ERROR_OPERATION_ABORTED};

            // Field order is the panic cleanup contract: dropping the gate first
            // wakes a gated worker; dropping the held write endpoint next gives
            // an already-entered read EOF; only then does the I/O guard cancel.
            // No process handle or Job exists, so Drop cannot kill the test host.
            struct HeldPipe<'a> {
                release: Option<mpsc::Sender<()>>,
                _held_write: File,
                io: IoCancellation<'a>,
            }

            let local_poison = AtomicBool::new(false);
            let stopped = Arc::new(AtomicBool::new(false));
            let (read, held_write) = pipe().expect("synthetic anonymous pipe");
            no_inherit(&read).expect("private read handle");
            no_inherit(&held_write).expect("private held endpoint");
            let mut read = read.into_file();
            let held_write = held_write.into_file();
            let (passed_tx, passed_rx) = mpsc::channel();
            let (release_tx, release_rx) = mpsc::channel();
            let (outcome_tx, outcome_rx) = mpsc::channel();
            let thread_stopped = stopped.clone();
            let worker = std::thread::Builder::new()
                .spawn(move || {
                    if thread_stopped.load(Ordering::SeqCst) {
                        return;
                    }
                    let _ = passed_tx.send(());
                    if release_rx.recv_timeout(Duration::from_secs(2)).is_err() {
                        return;
                    }
                    // Intentionally do not recheck stopped here: this is the
                    // controlled check-before-ReadFile race, with the opposite
                    // endpoint held open by the parent throughout finish().
                    let mut byte = [0u8; 1];
                    let aborted = read.read(&mut byte).is_err_and(|error| {
                        error.raw_os_error() == Some(ERROR_OPERATION_ABORTED as i32)
                    });
                    let _ = outcome_tx.send(aborted);
                })
                .expect("one owned synthetic I/O thread");
            let mut fixture = HeldPipe {
                release: Some(release_tx),
                _held_write: held_write,
                io: IoCancellation {
                    stopped: stopped.clone(),
                    threads: vec![worker],
                    job_to_terminate: None,
                    process: None,
                    poisoned: &local_poison,
                    settled: false,
                },
            };
            passed_rx
                .recv_timeout(Duration::from_secs(1))
                .expect("stop check passed before the held read");
            let first_cancel =
                unsafe { CancelSynchronousIo(fixture.io.threads[0].as_raw_handle()) };
            let first_error = unsafe { GetLastError() };
            assert_eq!(first_cancel, 0, "gate admits no pending pipe operation");
            assert_eq!(first_error, ERROR_NOT_FOUND);
            fixture
                .release
                .take()
                .expect("owned read gate")
                .send(())
                .expect("release the controlled read race");
            let started = Instant::now();
            assert!(fixture.io.finish().is_ok(), "owned held read must finish");
            assert!(started.elapsed() <= Duration::from_millis(IO_CLEANUP_MILLIS));
            assert!(fixture.io.threads.is_empty());
            assert!(stopped.load(Ordering::SeqCst));
            assert!(!local_poison.load(Ordering::SeqCst));
            assert!(outcome_rx.try_recv().expect("joined read result"));
        }

        fn fixture() -> JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
            let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = unsafe { zeroed() };
            limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_ACTIVE_PROCESS
                | JOB_OBJECT_LIMIT_PROCESS_MEMORY
                | JOB_OBJECT_LIMIT_JOB_MEMORY
                | JOB_OBJECT_LIMIT_PROCESS_TIME
                | JOB_OBJECT_LIMIT_JOB_TIME
                | JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            limits.BasicLimitInformation.ActiveProcessLimit = 1;
            limits.BasicLimitInformation.PerProcessUserTimeLimit =
                (WORKER_CPU_SECONDS * 10_000_000) as i64;
            limits.BasicLimitInformation.PerJobUserTimeLimit =
                (WORKER_CPU_SECONDS * 10_000_000) as i64;
            limits.ProcessMemoryLimit = WORKER_MEMORY_BYTES;
            limits.JobMemoryLimit = WORKER_MEMORY_BYTES;
            limits
        }
        #[test]
        fn limits_reject_zero_larger_missing_and_breakaway() {
            assert!(within_limits(&fixture()));
            let mut limits = fixture();
            limits.BasicLimitInformation.ActiveProcessLimit = 2;
            assert!(!within_limits(&limits));
            let mut limits = fixture();
            limits.ProcessMemoryLimit += 1;
            assert!(!within_limits(&limits));
            let mut limits = fixture();
            limits.JobMemoryLimit = 0;
            assert!(!within_limits(&limits));
            let mut limits = fixture();
            limits.BasicLimitInformation.PerProcessUserTimeLimit += 1;
            assert!(!within_limits(&limits));
            let mut limits = fixture();
            limits.BasicLimitInformation.PerJobUserTimeLimit = 0;
            assert!(!within_limits(&limits));
            let mut limits = fixture();
            limits.BasicLimitInformation.LimitFlags &= !JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
            assert!(!within_limits(&limits));
            for flag in [
                JOB_OBJECT_LIMIT_BREAKAWAY_OK,
                JOB_OBJECT_LIMIT_SILENT_BREAKAWAY_OK,
            ] {
                let mut limits = fixture();
                limits.BasicLimitInformation.LimitFlags |= flag;
                assert!(!within_limits(&limits));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn parent_cpu_proof() -> CpuProof {
        CpuProof {
            mechanism: CpuMechanism::ParentUserCpuWatchdog,
            trigger_user_100ns: WORKER_CPU_TICKS,
            trigger_wall_millis: 10_100,
            sampling_valid: true,
            live_child_observed: true,
            priority_rechecked: true,
            exact_job_dispatched: true,
            physical_expected_exit: true,
            cleanup_complete: true,
            terminal_denied: true,
            exact_marker: true,
            stats_valid: true,
            natural_exit: false,
            job_closed: false,
            worker_error: true,
            fallback_used: false,
        }
    }
    #[test]
    fn cpu_filetime_and_sampling_reject_signed_overflow_decrease_and_failed_queries() {
        assert_eq!(user_cpu_ticks(u32::MAX, 0).unwrap(), 4_294_967_295);
        assert_eq!(user_cpu_ticks(0, 1).unwrap(), 4_294_967_296);
        assert_eq!(user_cpu_ticks(u32::MAX, 0x7fff_ffff).unwrap(), i64::MAX);
        assert_eq!(
            user_cpu_ticks(0, 0x8000_0000),
            Err(ProcessingError::Isolation)
        );
        assert_eq!(
            user_cpu_ticks(u32::MAX, u32::MAX),
            Err(ProcessingError::Isolation)
        );
        for invalid in [Ok(-1), Err(ProcessingError::Isolation), Ok(0)] {
            let mut samples = CpuSampling::default();
            assert!(!samples.valid());
            assert_eq!(samples.observe(Ok(1)), Ok(1));
            assert_eq!(samples.observe(Ok(1)), Ok(1));
            assert_eq!(samples.observe(invalid), Err(ProcessingError::Isolation));
            assert!(!samples.valid());
            assert_eq!(samples.observe(Ok(2)), Err(ProcessingError::Isolation));
        }
    }
    #[test]
    fn cpu_threshold_priority_and_terminal_cause_are_closed_and_sticky() {
        assert_eq!(
            cpu_decision(Ok(WORKER_CPU_TICKS - 1), false, 14_999),
            TerminalCause::Running
        );
        assert_eq!(
            cpu_decision(Ok(WORKER_CPU_TICKS), false, 14_999),
            TerminalCause::Cpu
        );
        assert_eq!(
            cpu_decision(Ok(WORKER_CPU_TICKS), true, 14_999),
            TerminalCause::Cancel
        );
        assert_eq!(
            cpu_decision(Ok(WORKER_CPU_TICKS), false, 15_000),
            TerminalCause::Wall
        );
        assert_eq!(
            cpu_decision(Err(ProcessingError::Isolation), false, 1),
            TerminalCause::Other
        );
        assert_eq!(cpu_decision(Ok(-1), false, 1), TerminalCause::Other);
        for first in [
            TerminalCause::Cpu,
            TerminalCause::Wall,
            TerminalCause::Cancel,
            TerminalCause::Other,
        ] {
            let mut terminal = TerminalCause::Running;
            terminal.settle(first);
            for later in [
                TerminalCause::Running,
                TerminalCause::Cpu,
                TerminalCause::Other,
            ] {
                terminal.settle(later);
                assert_eq!(terminal, first);
            }
        }
    }
    #[test]
    fn cpu_parent_proof_requires_every_causal_field_and_never_a_wall_or_cleanup_kill() {
        let good = parent_cpu_proof();
        assert!(good.passes(true, Some(CPU_WATCHDOG_EXIT), WORKER_CPU_TICKS, 0));
        let reject = |proof: CpuProof| {
            assert!(!proof.passes(true, Some(CPU_WATCHDOG_EXIT), WORKER_CPU_TICKS, 0))
        };
        reject(CpuProof {
            mechanism: CpuMechanism::None,
            ..good
        });
        reject(CpuProof {
            trigger_user_100ns: WORKER_CPU_TICKS - 1,
            ..good
        });
        reject(CpuProof {
            trigger_user_100ns: WORKER_CPU_TICKS + 1,
            ..good
        });
        reject(CpuProof {
            trigger_wall_millis: 15_000,
            ..good
        });
        reject(CpuProof {
            sampling_valid: false,
            ..good
        });
        reject(CpuProof {
            live_child_observed: false,
            ..good
        });
        reject(CpuProof {
            priority_rechecked: false,
            ..good
        });
        reject(CpuProof {
            exact_job_dispatched: false,
            ..good
        });
        reject(CpuProof {
            physical_expected_exit: false,
            ..good
        });
        reject(CpuProof {
            cleanup_complete: false,
            ..good
        });
        reject(CpuProof {
            terminal_denied: false,
            ..good
        });
        reject(CpuProof {
            exact_marker: false,
            ..good
        });
        reject(CpuProof {
            stats_valid: false,
            ..good
        });
        reject(CpuProof {
            natural_exit: true,
            ..good
        });
        reject(CpuProof {
            job_closed: true,
            ..good
        });
        reject(CpuProof {
            worker_error: false,
            ..good
        });
        reject(CpuProof {
            fallback_used: true,
            ..good
        });
        assert!(!good.passes(false, Some(CPU_WATCHDOG_EXIT), WORKER_CPU_TICKS, 0));
        for exit in [None, Some(0), Some(1), Some(3)] {
            assert!(!good.passes(true, exit, WORKER_CPU_TICKS, 0));
        }
        assert!(!good.passes(true, Some(CPU_WATCHDOG_EXIT), WORKER_CPU_TICKS - 1, 0));
    }
    #[test]
    fn cpu_native_proof_retains_the_actual_native_counter_and_original_exit_requirements() {
        let good = CpuProof {
            mechanism: CpuMechanism::NativeJobCpuLimit,
            trigger_user_100ns: 0,
            trigger_wall_millis: 0,
            live_child_observed: false,
            priority_rechecked: false,
            exact_job_dispatched: false,
            physical_expected_exit: false,
            natural_exit: true,
            ..parent_cpu_proof()
        };
        assert!(good.passes(true, Some(1), WORKER_CPU_TICKS, 1));
        assert!(!good.passes(true, Some(1), WORKER_CPU_TICKS, 0));
        assert!(!good.passes(true, Some(0), WORKER_CPU_TICKS, 1));
        assert!(!good.passes(true, None, WORKER_CPU_TICKS, 1));
        assert!(!CpuProof {
            natural_exit: false,
            ..good
        }
        .passes(true, Some(1), WORKER_CPU_TICKS, 1));
        assert!(!CpuProof {
            worker_error: false,
            ..good
        }
        .passes(true, Some(1), WORKER_CPU_TICKS, 1));
        assert!(!CpuProof {
            exact_job_dispatched: true,
            ..good
        }
        .passes(true, Some(1), WORKER_CPU_TICKS, 1));
    }
    #[test]
    fn cpu_receipt_serialises_only_the_fixed_causal_schema_and_real_native_counter() {
        let result = ResourceProbeResult {
            code: "cpu-kill",
            status: "PASS",
            reason: "observed-synthetic-resource-containment",
            elapsed_millis: 10_150,
            exit_code: Some(CPU_WATCHDOG_EXIT),
            cpu_user_100ns: WORKER_CPU_TICKS,
            peak_memory_bytes: 1_638_400,
            limit_terminations: 0,
            boundary_available: true,
            cpu_proof: Some(parent_cpu_proof()),
        };
        let body = serde_json::to_value(result).unwrap();
        assert_eq!(body["limitTerminations"], 0);
        assert_eq!(body["cpuUser100ns"], WORKER_CPU_TICKS);
        assert_eq!(body["exitCode"], CPU_WATCHDOG_EXIT);
        assert_eq!(body.as_object().unwrap().len(), 10);
        let proof = body["cpuProof"].as_object().unwrap();
        let expected = [
            "mechanism",
            "triggerUser100ns",
            "triggerWallMillis",
            "samplingValid",
            "liveChildObserved",
            "priorityRechecked",
            "exactJobDispatched",
            "physicalExpectedExit",
            "cleanupComplete",
            "terminalDenied",
            "exactMarker",
            "statsValid",
            "naturalExit",
            "jobClosed",
            "workerError",
            "fallbackUsed",
        ];
        assert_eq!(proof.len(), expected.len());
        assert!(proof.keys().all(|key| expected.contains(&key.as_str())));
        assert_eq!(proof["mechanism"], "parent-user-cpu-watchdog");
        assert_eq!(proof["triggerUser100ns"], WORKER_CPU_TICKS);
    }
    #[test]
    fn native_failure_exit_codes_are_pdf_codec_only_and_fail_closed() {
        for stage in PdfFailureStage::ALL {
            let code = worker_failure_exit(ProcessingError::Pdf(stage));
            assert_ne!(code, 0);
            assert_eq!(
                worker_exit_error(InputKind::Pdf, true, code),
                ProcessingError::Pdf(stage)
            );
            for kind in [
                InputKind::Text,
                InputKind::Png,
                InputKind::Jpeg,
                InputKind::Gif,
            ] {
                assert_eq!(worker_exit_error(kind, true, code), ProcessingError::Worker);
            }
            assert_eq!(
                worker_exit_error(InputKind::Pdf, false, code),
                ProcessingError::Worker
            );
        }
        for error in [
            ProcessingError::Invalid,
            ProcessingError::Limit,
            ProcessingError::Unsupported,
            ProcessingError::Isolation,
            ProcessingError::Cancelled,
            ProcessingError::Timeout,
            ProcessingError::Worker,
        ] {
            assert_eq!(worker_failure_exit(error), 3);
        }
        for code in [0, 1, 3, 63, 82, 255, u32::MAX] {
            assert_eq!(
                worker_exit_error(InputKind::Pdf, true, code),
                ProcessingError::Worker
            );
        }
    }

    #[test]
    fn closed_request_refuses_unknown_suffix_and_oversized_lengths() {
        let mut request = Vec::new();
        write_request(&mut request, InputKind::Text, b"synthetic").unwrap();
        assert_eq!(
            read_request(&mut request.as_slice()).unwrap().1.as_slice(),
            b"synthetic"
        );
        request.push(0);
        assert!(matches!(
            read_request(&mut request.as_slice()),
            Err(ProcessingError::Invalid)
        ));
        request[8] = 99;
        assert!(matches!(
            read_request(&mut request.as_slice()),
            Err(ProcessingError::Invalid)
        ));
        let mut oversized = [0u8; 13];
        oversized[..8].copy_from_slice(INPUT_MAGIC);
        oversized[8] = InputKind::Pdf as u8;
        oversized[9..13].copy_from_slice(&((MAX_SOURCE_BYTES + 1) as u32).to_le_bytes());
        assert!(matches!(
            read_request(&mut oversized.as_slice()),
            Err(ProcessingError::Limit)
        ));
    }
    #[test]
    fn text_witness_and_payload_cannot_be_changed_or_extended() {
        let original = b"synthetic public fixture";
        let processed =
            attachment_processing::process_in_worker(InputKind::Text, original).unwrap();
        let mut output = Vec::new();
        encode_response(&mut output, InputKind::Text, original, &processed).unwrap();
        assert!(decode_response(InputKind::Text, original, &output).is_ok());
        assert!(decode_response(InputKind::Text, b"different", &output).is_err());
        let end = output.len() - 1;
        output[end] ^= 1;
        assert!(decode_response(InputKind::Text, original, &output).is_err());
        output[end] ^= 1;
        output.push(0);
        assert!(decode_response(InputKind::Text, original, &output).is_err());
    }
    #[test]
    fn visual_response_is_reencoded_and_exactly_bound() {
        let page = attachment_processing::page_from_rgba(1, 1, vec![17, 33, 65, 255]).unwrap();
        let original = page.png.clone();
        let processed = ProcessedAttachment {
            source_sha256: attachment_processing::source_hash(&original),
            content: ProcessedContent::Pages(vec![page]),
        };
        let mut output = Vec::new();
        encode_response(&mut output, InputKind::Png, &original, &processed).unwrap();
        let result = decode_response(InputKind::Png, &original, &output).unwrap();
        let ProcessedContent::Pages(pages) = result.content else {
            panic!("pages")
        };
        assert_eq!(pages[0].rgba, vec![17, 33, 65, 255]);
        let last = output.len() - 1;
        output[last] ^= 1;
        assert!(matches!(
            decode_response(InputKind::Png, &original, &output),
            Err(ProcessingError::Invalid)
        ));
    }
    #[test]
    fn cancellation_denies_before_child_creation() {
        let cancelled = AtomicBool::new(true);
        assert!(matches!(
            run_isolated(InputKind::Text, b"synthetic", &cancelled),
            Err(ProcessingError::Cancelled | ProcessingError::Isolation)
        ));
    }
}
