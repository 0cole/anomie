use std::{fs, io};

use log::{debug, info};

use crate::errors::{self, ExitStatus};

fn save_crash(
    report_path: &str,
    crash_id: usize,
    input: &[u8],
    crash_type: &str,
) -> io::Result<()> {
    let path = format!("{report_path}/{crash_type}/crash-{crash_id}.bin");
    debug!("Recording crash at {path:?}");
    fs::write(path, input)?;
    Ok(())
}

pub fn analyze_result(report_path: &str, result: ExitStatus, crash_id: usize, input: &[u8]) {
    match result {
        ExitStatus::ExitCode(code) => {
            debug!("Process exited gracefully with code {code}");
        }
        ExitStatus::Signal(sig) => {
            let (signal_desc, signal) = match sig {
                errors::SIGILL => ("illegal instruction", "SIGILL"),
                errors::SIGABRT => ("abort function", "SIGABRT"),
                errors::SIGFPE => ("floating point exception", "SIGFPE"),
                errors::SIGSEGV => ("segmentation fault", "SIGSEGV"),
                errors::SIGPIPE => ("pipe error", "SIGPIPE"),
                errors::SIGTERM => ("termination error", "SIGTERM"),
                _ => ("unknown error", "UNKNOWN"),
            };
            info!("Hit! Process crashed due to a {signal_desc} error ({signal})");
            save_crash(report_path, crash_id, input, signal).unwrap();
        }
        // ExitStatus::Timeout => {
        //     info!("Hit! Process timed out");
        // }
        ExitStatus::Error(msg) => {
            info!("Hit! Process execution error: {msg}");
        }
    }
}
