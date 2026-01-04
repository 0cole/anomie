use anyhow::Result;
use log::{debug, info};
use serde::Serialize;
use std::{
    fs::{self, remove_file},
    io::Read,
    path::PathBuf,
};

use crate::{
    errors::{self, ExitStatus},
    types::{CrashStats, StructuredInput},
};

#[derive(Serialize)]
pub struct CrashAnalyzer {
    pub crashes: Vec<Crash>,
    pub report_path: PathBuf,
    pub stats: CrashStats,
}

#[derive(Serialize)]
pub struct Crash {
    pub file: String,
    pub mutations: Vec<String>,
}

impl CrashAnalyzer {
    pub fn new(report_path: PathBuf) -> Self {
        let stats = CrashStats {
            total: 0,
            sigill: 0,
            sigabrt: 0,
            sigfpe: 0,
            sigsegv: 0,
            sigpipe: 0,
            sigterm: 0,
            timeout: 0,
        };

        let crashes: Vec<Crash> = Vec::new();
        Self {
            crashes,
            report_path,
            stats,
        }
    }

    pub fn analyze(
        &mut self,
        crash_id: usize,
        result: ExitStatus,
        input: StructuredInput,
        mutation_array: Vec<String>,
    ) -> Result<()> {
        let mut name: &str = "";
        let mut crash_occurred = false;

        match result {
            ExitStatus::ExitCode(code) => {
                debug!("Process exited gracefully with code {code}");
                if let StructuredInput::FileInput { path, .. } = &input {
                    remove_file(path)?;
                }
            }
            ExitStatus::Signal(sig) => {
                let desc: &str;
                (desc, name) = match sig {
                    errors::SIGILL => {
                        self.stats.sigill += 1;
                        ("illegal instruction", "SIGILL")
                    }
                    errors::SIGABRT => {
                        self.stats.sigabrt += 1;
                        ("abort function", "SIGABRT")
                    }
                    errors::SIGFPE => {
                        self.stats.sigfpe += 1;
                        ("floating point exception", "SIGFPE")
                    }
                    errors::SIGSEGV => {
                        self.stats.sigsegv += 1;
                        ("segmentation fault", "SIGSEGV")
                    }
                    errors::SIGPIPE => {
                        self.stats.sigpipe += 1;
                        ("pipe error", "SIGPIPE")
                    }
                    errors::SIGTERM => {
                        self.stats.sigterm += 1;
                        ("termination error", "SIGTERM")
                    }
                    _ => ("unknown error", "UNKNOWN"),
                };

                info!(
                    "Hit! Process crashed due to a {desc} error ({name}). Recording in {}/{name}/ as crash-{crash_id}",
                    self.report_path.display()
                );
                crash_occurred = true;
            }
            ExitStatus::Timeout(limit) => {
                self.stats.timeout += 1;
                info!("Hit! Process timed out after exceeding {limit} ms");
                crash_occurred = true;
            }
            ExitStatus::Error(msg) => {
                info!("Hit! Process execution error: {msg}");
                crash_occurred = true;
            }
        }

        if crash_occurred {
            self.record_crash(crash_id, input, name, mutation_array)?;
            self.stats.total += 1;
        }

        Ok(())
    }

    pub fn record_crash(
        &mut self,
        crash_id: usize,
        input: StructuredInput,
        crash_type: &str,
        mutation_array: Vec<String>,
    ) -> Result<()> {
        let (output_path, bytes) = match input {
            StructuredInput::StringInput(bytes) => {
                let output_path = format!(
                    "{}/{crash_type}/crash-{crash_id}.bin",
                    self.report_path.display()
                );
                (output_path, bytes)
            }
            StructuredInput::FileInput { path, extension } => {
                let output_path = format!(
                    "{}/{crash_type}/crash-{crash_id}.{extension}",
                    self.report_path.display()
                );
                let mut bytes = Vec::new();
                fs::File::open(path)?.read_to_end(&mut bytes)?;
                (output_path, bytes)
            }
        };

        debug!("Recording crash at {output_path}");
        fs::write(&output_path, bytes)?;

        let crash = Crash {
            file: output_path,
            mutations: mutation_array,
        };
        self.crashes.push(crash);

        Ok(())
    }
}
