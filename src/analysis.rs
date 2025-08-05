use std::{
    fs,
    io::{self, Read},
};

use log::{debug, info};

use crate::{
    errors::{self, ExitStatus},
    types::StructuredInput,
};

fn save_crash(
    report_path: &str,
    crash_id: usize,
    input: StructuredInput,
    crash_type: &str,
) -> io::Result<()> {
    match input {
        StructuredInput::StringInput(bytes) => {
            let path = format!("{report_path}/{crash_type}/crash-{crash_id}.bin");
            debug!("Recording string-based crash at {path:?}");
            fs::write(path, bytes)
        }
        StructuredInput::FileInput(file_path, ext) => {
            let output_path = format!("{report_path}/{crash_type}/crash-{crash_id}.{ext}");
            debug!("Recording file-based crash at {output_path:?}");

            let mut contents = Vec::new();
            fs::File::open(&file_path)?.read_to_end(&mut contents)?;
            fs::write(output_path, contents)
        }
    }
}

pub fn analyze_result(
    report_path: &str,
    result: ExitStatus,
    crash_id: usize,
    input: StructuredInput,
) {
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
            info!(
                "Hit! Process crashed due to a {signal_desc} error ({signal}). Recording in {report_path}/{signal}/ as crash-{crash_id}"
            );
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
