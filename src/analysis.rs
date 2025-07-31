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
        ExitStatus::Signal(sig) => match sig {
            errors::SIGILL => {
                info!("Hit! Process encountered an illegal instruction");
                save_crash(report_path, crash_id, input, "SIGILL").unwrap();
            }
            errors::SIGABRT => {
                info!("Hit! Process executed the abort function");
                save_crash(report_path, crash_id, input, "SIGABRT").unwrap();
            }
            errors::SIGFPE => {
                info!("Hit! Process encountered a floating point exception");
                save_crash(report_path, crash_id, input, "SIGFPE").unwrap();
            }
            errors::SIGSEGV => {
                info!("Hit! Process encountered a segmentation fault");
                save_crash(report_path, crash_id, input, "SIGSEGV").unwrap();
            }
            errors::SIGPIPE => {
                info!("Hit! Process encountered a pipe error");
                save_crash(report_path, crash_id, input, "SIGPIPE").unwrap();
            }
            errors::SIGTERM => {
                info!("Hit! Process was terminated");
                save_crash(report_path, crash_id, input, "SIGTERM").unwrap();
            }
            _ => {
                info!("Hit! Process crashed with unknown signal {sig}");
                save_crash(report_path, crash_id, input, "UKNOWN").unwrap();
            }
        },
        // ExitStatus::Timeout => {
        //     info!("Hit! Process timed out");
        // }
        ExitStatus::Error(msg) => {
            info!("Hit! Process execution error: {msg}");
        }
    }
}
