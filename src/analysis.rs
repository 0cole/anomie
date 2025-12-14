use anyhow::Result;
use log::{debug, info};
use std::{fs, io::Read};

use crate::{
    errors::{self, ExitStatus},
    types::{RunDetails, StructuredInput},
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
        StructuredInput::FileInput(file_path, ext) => {
            let output_path = format!("{report_path}/{crash_type}/crash-{crash_id}.{ext}");
            debug!("Recording file-based crash at {output_path:?}");

            let mut contents = Vec::new();
            fs::File::open(&file_path)?.read_to_end(&mut contents)?;
            fs::write(output_path, contents)?;
        }
    }
    Ok(())
}

pub fn analyze_result(
    report_path: &str,
    run_details: &mut RunDetails,
    result: ExitStatus,
    crash_id: usize,
    input: StructuredInput,
) -> Result<()> {
    match result {
        ExitStatus::ExitCode(code) => {
            debug!("Process exited gracefully with code {code}");
        }
        ExitStatus::Signal(sig) => {
            let (signal_desc, signal) = match sig {
                errors::SIGILL => {
                    run_details.sigill_hits += 1;
                    ("illegal instruction", "SIGILL")
                }
                errors::SIGABRT => {
                    run_details.sigabrt_hits += 1;
                    ("abort function", "SIGABRT")
                }
                errors::SIGFPE => {
                    run_details.sigfpe_hits += 1;
                    ("floating point exception", "SIGFPE")
                }
                errors::SIGSEGV => {
                    run_details.sigsegv_hits += 1;
                    ("segmentation fault", "SIGSEGV")
                }
                errors::SIGPIPE => {
                    run_details.sigpipe_hits += 1;
                    ("pipe error", "SIGPIPE")
                }
                errors::SIGTERM => {
                    run_details.sigterm_hits += 1;
                    ("termination error", "SIGTERM")
                }
                _ => ("unknown error", "UNKNOWN"),
            };
            info!(
                "Hit! Process crashed due to a {signal_desc} error ({signal}). Recording in {report_path}/{signal}/ as crash-{crash_id}"
            );
            save_crash(report_path, crash_id, input, signal)?;
            run_details.total_hits += 1;
        }
        ExitStatus::Timeout(limit) => {
            run_details.timeout_hits += 1;
            info!("Hit! Process timed out after exceeding {limit} ms");
            save_crash(report_path, crash_id, input, "TIMEOUT")?;
            run_details.total_hits += 1;
        }
        ExitStatus::Error(msg) => {
            info!("Hit! Process execution error: {msg}");
        }
    }
    Ok(())
}
