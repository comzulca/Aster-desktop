//! Windows creates the pinned decoder inside its already-limited Job Object.
use super::{system_directory, VerifiedAssets, FAILED, MODEL, UNAVAILABLE};
use std::{
    ffi::{OsStr, OsString},
    fs::OpenOptions,
    io::{pipe, PipeReader, PipeWriter},
    mem::{size_of, size_of_val},
    os::windows::{
        ffi::OsStrExt,
        io::{AsRawHandle, FromRawHandle, OwnedHandle},
    },
    ptr::{null, null_mut},
};
use windows_sys::Win32::{
    Foundation::{
        GetLastError, SetHandleInformation, ERROR_ACCESS_DENIED, ERROR_NOT_ENOUGH_QUOTA,
        HANDLE_FLAG_INHERIT, WAIT_OBJECT_0, WAIT_TIMEOUT,
    },
    System::{
        JobObjects::{
            CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject,
            JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_ACTIVE_PROCESS,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE, JOB_OBJECT_LIMIT_PROCESS_MEMORY,
        },
        Threading::{
            CreateProcessW, DeleteProcThreadAttributeList, GetExitCodeProcess,
            InitializeProcThreadAttributeList, UpdateProcThreadAttribute, WaitForSingleObject,
            CREATE_NO_WINDOW, CREATE_UNICODE_ENVIRONMENT, EXTENDED_STARTUPINFO_PRESENT, INFINITE,
            LPPROC_THREAD_ATTRIBUTE_LIST, PROCESS_INFORMATION, PROC_THREAD_ATTRIBUTE_HANDLE_LIST,
            PROC_THREAD_ATTRIBUTE_JOB_LIST, STARTF_USESTDHANDLES, STARTUPINFOEXW, STARTUPINFOW,
        },
    },
};

pub(super) struct Decoder {
    process: OwnedHandle,
    job: Option<OwnedHandle>,
}

impl Decoder {
    pub(super) fn start(
        assets: &VerifiedAssets,
    ) -> Result<(Self, PipeWriter, PipeReader), &'static str> {
        let job = limited_job()?;
        let (process, input, output) = spawn_in_job(assets, &job)?;
        Ok((
            Self {
                process,
                job: Some(job),
            },
            input,
            output,
        ))
    }

    pub(super) fn try_wait(&self) -> Result<Option<u32>, &'static str> {
        let handle = self.process.as_raw_handle();
        match unsafe { WaitForSingleObject(handle, 0) } {
            WAIT_TIMEOUT => Ok(None),
            WAIT_OBJECT_0 => {
                let mut code = 0;
                if unsafe { GetExitCodeProcess(handle, &mut code) } == 0 {
                    Err(FAILED)
                } else {
                    Ok(Some(code))
                }
            }
            _ => Err(FAILED),
        }
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        // Neither the child nor any other owner holds a copy of the Job handle.
        // Kill-on-close also covers an early return before the pipe threads start.
        drop(self.job.take());
        unsafe { WaitForSingleObject(self.process.as_raw_handle(), INFINITE) };
    }
}

fn limited_job() -> Result<OwnedHandle, &'static str> {
    let raw = unsafe { CreateJobObjectW(null(), null()) };
    if raw.is_null() {
        return Err(FAILED);
    }
    let job = unsafe { OwnedHandle::from_raw_handle(raw) };
    let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
        | JOB_OBJECT_LIMIT_ACTIVE_PROCESS
        | JOB_OBJECT_LIMIT_PROCESS_MEMORY;
    limits.BasicLimitInformation.ActiveProcessLimit = 1;
    limits.ProcessMemoryLimit = 1024 * 1024 * 1024;
    if unsafe {
        SetInformationJobObject(
            raw,
            JobObjectExtendedLimitInformation,
            (&limits as *const JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
            size_of_val(&limits) as u32,
        )
    } == 0
    {
        return Err(FAILED);
    }
    Ok(job)
}

fn wide_z(value: &OsStr) -> Result<Vec<u16>, &'static str> {
    let mut units: Vec<_> = value.encode_wide().collect();
    if units.is_empty() || units.contains(&0) {
        return Err(UNAVAILABLE);
    }
    units.push(0);
    Ok(units)
}

struct AttributeList(LPPROC_THREAD_ATTRIBUTE_LIST);

impl Drop for AttributeList {
    fn drop(&mut self) {
        unsafe { DeleteProcThreadAttributeList(self.0) };
    }
}

fn spawn_in_job(
    assets: &VerifiedAssets,
    job: &OwnedHandle,
) -> Result<(OwnedHandle, PipeWriter, PipeReader), &'static str> {
    if !assets.directory.is_absolute() {
        return Err(UNAVAILABLE);
    }
    let directory = wide_z(assets.directory.as_os_str())?;
    let executable = wide_z(assets.directory.join("whisper-cli.exe").as_os_str())?;
    let mut command: Vec<u16> =
        format!("whisper-cli.exe -m {MODEL} -f - -l auto -otxt -of - -np -nt -ng -t 4\0")
            .encode_utf16()
            .collect();
    let system = system_directory()?;
    let mut path_entry = OsString::from("PATH=");
    path_entry.push(&system);
    let mut root_entry = OsString::from("SystemRoot=");
    root_entry.push(system.parent().ok_or(UNAVAILABLE)?);
    let mut environment = wide_z(&path_entry)?;
    environment.extend(wide_z(&root_entry)?);
    environment.push(0);

    let (child_input, input) = pipe().map_err(|_| FAILED)?;
    let (output, child_output) = pipe().map_err(|_| FAILED)?;
    let child_error = OpenOptions::new()
        .write(true)
        .open(r"\\.\NUL")
        .map_err(|_| FAILED)?;
    let stdio = [
        child_input.as_raw_handle(),
        child_output.as_raw_handle(),
        child_error.as_raw_handle(),
    ];
    for &handle in &stdio {
        if unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT, HANDLE_FLAG_INHERIT) } == 0 {
            return Err(FAILED);
        }
    }

    // Windows retains the attribute values through list deletion. Declare the
    // arrays and aligned backing storage before the list's drop guard.
    let jobs = [job.as_raw_handle()];
    let mut bytes = 0;
    unsafe { InitializeProcThreadAttributeList(null_mut(), 2, 0, &mut bytes) };
    if bytes == 0 || bytes > 64 * 1024 {
        return Err(FAILED);
    }
    let mut storage = vec![0usize; bytes.div_ceil(size_of::<usize>())];
    let raw = storage.as_mut_ptr().cast();
    if unsafe { InitializeProcThreadAttributeList(raw, 2, 0, &mut bytes) } == 0 {
        return Err(FAILED);
    }
    let attributes = AttributeList(raw);
    for (attribute, handles) in [
        (PROC_THREAD_ATTRIBUTE_HANDLE_LIST, stdio.as_slice()),
        (PROC_THREAD_ATTRIBUTE_JOB_LIST, jobs.as_slice()),
    ] {
        if unsafe {
            UpdateProcThreadAttribute(
                attributes.0,
                0,
                attribute as usize,
                handles.as_ptr().cast(),
                size_of_val(handles),
                null_mut(),
                null(),
            )
        } == 0
        {
            return Err(FAILED);
        }
    }
    let startup = STARTUPINFOEXW {
        StartupInfo: STARTUPINFOW {
            cb: size_of::<STARTUPINFOEXW>() as u32,
            dwFlags: STARTF_USESTDHANDLES,
            hStdInput: stdio[0],
            hStdOutput: stdio[1],
            hStdError: stdio[2],
            ..Default::default()
        },
        lpAttributeList: attributes.0,
    };
    let mut result = PROCESS_INFORMATION::default();
    if unsafe {
        CreateProcessW(
            executable.as_ptr(),
            command.as_mut_ptr(),
            null(),
            null(),
            1,
            CREATE_UNICODE_ENVIRONMENT | CREATE_NO_WINDOW | EXTENDED_STARTUPINFO_PRESENT,
            environment.as_ptr().cast(),
            directory.as_ptr(),
            &startup.StartupInfo,
            &mut result,
        )
    } == 0
    {
        return Err(match unsafe { GetLastError() } {
            ERROR_ACCESS_DENIED | ERROR_NOT_ENOUGH_QUOTA => FAILED,
            _ => "speech_runtime",
        });
    }
    // Both returned handles are owned by the caller on success. The executable
    // is already in the job, and no microphone bytes have crossed the pipe.
    let process = unsafe { OwnedHandle::from_raw_handle(result.hProcess) };
    drop(unsafe { OwnedHandle::from_raw_handle(result.hThread) });
    Ok((process, input, output))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use windows_sys::Win32::System::JobObjects::{IsProcessInJob, QueryInformationJobObject};

    #[test]
    fn decoder_starts_in_limited_job_and_job_close_stops_it_without_audio() {
        let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../reports/speech");
        let assets = VerifiedAssets::open(&directory).unwrap();
        let (worker, _input, _output) = Decoder::start(&assets).unwrap();
        let job = worker.job.as_ref().unwrap();
        let process = worker.process.try_clone().unwrap();
        let mut member = 0;
        let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        assert_ne!(
            unsafe { IsProcessInJob(process.as_raw_handle(), job.as_raw_handle(), &mut member) },
            0
        );
        assert_ne!(member, 0);
        assert_ne!(
            unsafe {
                QueryInformationJobObject(
                    job.as_raw_handle(),
                    JobObjectExtendedLimitInformation,
                    (&mut limits as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                    size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
                    null_mut(),
                )
            },
            0
        );
        assert_eq!(limits.BasicLimitInformation.ActiveProcessLimit, 1);
        assert_eq!(limits.ProcessMemoryLimit, 1024 * 1024 * 1024);
        assert_eq!(
            limits.BasicLimitInformation.LimitFlags,
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE
                | JOB_OBJECT_LIMIT_ACTIVE_PROCESS
                | JOB_OBJECT_LIMIT_PROCESS_MEMORY
        );
        // The original worker is waiting for input. A full Job refuses another
        // launch, with no retry outside the Job; the first worker still lives.
        assert!(spawn_in_job(&assets, job).is_err());
        assert_eq!(worker.try_wait(), Ok(None));
        drop(worker);
        assert_eq!(
            unsafe { WaitForSingleObject(process.as_raw_handle(), 0) },
            WAIT_OBJECT_0
        );
    }
}
