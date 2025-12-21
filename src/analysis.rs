use anyhow::Result;
use log::{debug, info};
use std::{
    fs::{self, remove_file},
    io::Read,
};

use crate::{
    errors::{self, ExitStatus},
    types::{CrashStats, StructuredInput},
};

fn save_crash(
    report_path: &str,
    crash_id: usize,
    input: StructuredInput,
    crash_type: &str,
) -> Result<()> {
    match input {
        StructuredInput::StringInput(bytes) => {
            let path = format!("{report_path}/{crash_type}/crash-{crash_id}.bin");
            debug!("Recording string-based crash at {path:?}");
            fs::write(path, bytes)?;
        }
        StructuredInput::FileInput { path, extension } => {
            let output_path = format!("{report_path}/{crash_type}/crash-{crash_id}.{extension}");
            debug!("Recording file-based crash at {output_path:?}");

            let mut contents = Vec::new();
            fs::File::open(path)?.read_to_end(&mut contents)?;
            fs::write(output_path, contents)?;
        }
    }
    Ok(())
}

pub fn analyze_result(
    report_path: &str,
    crash_stats: &mut CrashStats,
    result: ExitStatus,
    crash_id: usize,
    input: StructuredInput,
) -> Result<()> {
    match result {
        ExitStatus::ExitCode(code) => {
            debug!("Process exited gracefully with code {code}");
            if let StructuredInput::FileInput { path, .. } = input {
                debug!(
                    "no crashes occurred, removing {}",
                    path.as_path().to_string_lossy().into_owned()
                );
                remove_file(&path)?;
            }
        }
        ExitStatus::Signal(sig) => {
            let (signal_desc, signal) = match sig {
                errors::SIGILL => {
                    crash_stats.sigill += 1;
                    ("illegal instruction", "SIGILL")
                }
                errors::SIGABRT => {
                    crash_stats.sigabrt += 1;
                    ("abort function", "SIGABRT")
                }
                errors::SIGFPE => {
                    crash_stats.sigfpe += 1;
                    ("floating point exception", "SIGFPE")
                }
                errors::SIGSEGV => {
                    crash_stats.sigsegv += 1;
                    ("segmentation fault", "SIGSEGV")
                }
                errors::SIGPIPE => {
                    crash_stats.sigpipe += 1;
                    ("pipe error", "SIGPIPE")
                }
                errors::SIGTERM => {
                    crash_stats.sigterm += 1;
                    ("termination error", "SIGTERM")
                }
                _ => ("unknown error", "UNKNOWN"),
            };
            info!(
                "Hit! Process crashed due to a {signal_desc} error ({signal}). Recording in {report_path}/{signal}/ as crash-{crash_id}"
            );
            save_crash(report_path, crash_id, input, signal)?;
            crash_stats.total += 1;
        }
        ExitStatus::Timeout(limit) => {
            crash_stats.timeout += 1;
            info!("Hit! Process timed out after exceeding {limit} ms");
            save_crash(report_path, crash_id, input, "TIMEOUT")?;
            crash_stats.total += 1;
        }
        ExitStatus::Error(msg) => {
            info!("Hit! Process execution error: {msg}");
        }
    }
    Ok(())
}
