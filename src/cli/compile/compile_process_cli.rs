// SPDX-License-Identifier: MPL-2.0
use eyre::Context;
use eyre::Result;
use eyre::eyre;
use std::io::Read;
use std::process::Child;
use std::process::Command;
use std::process::ExitStatus;
use std::process::Output;
use std::process::Stdio;
use std::thread;
use std::time::Duration;
use teamy_cancellation::CancellationToken;

#[cfg(windows)]
fn process_job() -> Result<windows::core::Owned<windows::Win32::Foundation::HANDLE>> {
    use windows::Win32::System::JobObjects::CreateJobObjectW;
    use windows::Win32::System::JobObjects::JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    use windows::Win32::System::JobObjects::JOBOBJECT_EXTENDED_LIMIT_INFORMATION;
    use windows::Win32::System::JobObjects::JobObjectExtendedLimitInformation;
    use windows::Win32::System::JobObjects::SetInformationJobObject;
    use windows::core::Owned;
    // SAFETY: Null attributes and name request a new anonymous job.
    let handle = unsafe { CreateJobObjectW(None, None) }?;
    // SAFETY: Successful creation transferred this unique kernel handle.
    let job = unsafe { Owned::new(handle) };
    let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
    limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
    // SAFETY: The live job and correctly sized, initialized limit structure
    // remain valid throughout this non-retaining configuration call.
    unsafe {
        SetInformationJobObject(
            *job,
            JobObjectExtendedLimitInformation,
            (&raw const limits).cast(),
            u32::try_from(std::mem::size_of_val(&limits))?,
        )
    }?;
    Ok(job)
}

#[cfg(windows)]
fn assign_job(job: windows::Win32::Foundation::HANDLE, child: &Child) -> Result<()> {
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::Foundation::HANDLE;
    use windows::Win32::System::JobObjects::AssignProcessToJobObject;
    // SAFETY: Both handles are borrowed from their live owners. Assignment
    // transfers no handle ownership. Ordinary compiler descendants inherit it.
    unsafe { AssignProcessToJobObject(job, HANDLE(child.as_raw_handle())) }
        .wrap_err("cannot contain compiler/build process in a Windows job")
}

pub(super) fn run_process(
    command: &mut Command,
    label: &str,
    cancellation: &CancellationToken,
) -> Result<()> {
    let output =
        capture_process(command, cancellation).wrap_err_with(|| format!("cannot start {label}"))?;
    if !output.status.success() {
        return Err(eyre!(
            "{label} failed ({}):\n{}{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ));
    }
    Ok(())
}

pub(super) fn capture_process(
    command: &mut Command,
    cancellation: &CancellationToken,
) -> Result<Output> {
    cancellation.bail_if_cancelled()?;
    #[cfg(windows)]
    let job = {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
        process_job()?
    };
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .wrap_err("cannot launch compiler/build process")?;
    #[cfg(windows)]
    if let Err(error) = assign_job(*job, &child) {
        terminate_child(&mut child);
        return Err(error);
    }
    // Stable std::process starts the child before job assignment. This owns
    // normal compiler/linker descendants, not adversarial pre-assignment forks.
    let stdout = child.stdout.take().expect("requested compiler stdout pipe");
    let stderr = child.stderr.take().expect("requested compiler stderr pipe");
    let stdout = thread::spawn(move || read_diagnostics(stdout));
    let stderr = thread::spawn(move || read_diagnostics(stderr));
    let status = wait_child(&mut child, cancellation);
    #[cfg(windows)]
    drop(job); // Close descendant pipes before joining diagnostic readers.
    if status.is_err() {
        terminate_child(&mut child);
    }
    let stdout = stdout
        .join()
        .map_err(|_panic| eyre!("compiler stdout reader panicked"))??;
    let stderr = stderr
        .join()
        .map_err(|_panic| eyre!("compiler stderr reader panicked"))??;
    Ok(Output {
        status: status?,
        stdout,
        stderr,
    })
}

fn wait_child(child: &mut Child, cancellation: &CancellationToken) -> Result<ExitStatus> {
    loop {
        if let Some(status) = child
            .try_wait()
            .wrap_err("cannot poll compiler/build process")?
        {
            return Ok(status);
        }
        cancellation.bail_if_cancelled()?;
        thread::sleep(Duration::from_millis(10));
    }
}

fn terminate_child(child: &mut Child) {
    let _killed = child.kill();
    let _reaped = child.wait();
}

fn read_diagnostics(mut input: impl Read) -> std::io::Result<Vec<u8>> {
    const MAX_DIAGNOSTICS: usize = 1024 * 1024;
    let mut result = Vec::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        let retained = count.min(MAX_DIAGNOSTICS.saturating_sub(result.len()));
        result.extend_from_slice(&buffer[..retained]);
    }
    Ok(result)
}
