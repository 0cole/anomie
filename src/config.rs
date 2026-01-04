use anyhow::{Result, anyhow};
use clap::Parser;
use rand::{SeedableRng, rngs::SmallRng};
use std::fs;
use tempfile::tempdir;

use crate::types::{Config, CrashStats, FuzzType};

#[derive(Parser, Debug)]
pub struct RawConfig {
    #[arg(short, long)]
    pub bin_path: String,

    #[arg(long, default_value = "string")]
    pub fuzz_type: String,

    #[arg(long, default_value_t = 1000)]
    pub max_iterations: usize,

    #[arg(long, default_value_t = 100)]
    pub timeout: u64,

    #[arg(short, long, default_value = "./reports")]
    pub report_path: String,

    #[arg(short, long)]
    pub seed: Option<u64>,

    // everything after is part of args
    #[arg(last = true)]
    pub bin_args: String,
}

impl RawConfig {
    pub fn validate(&self) -> Result<Config, anyhow::Error> {
        // validate the binary passed in
        let metadata = fs::metadata(&self.bin_path)
            .map_err(|_| anyhow!("invalid binary path, double check the path exists"))?;
        if !metadata.is_file() {
            return Err(anyhow!("path does not correspond to a binary"));
        }

        // validate the type passed in
        let validated_fuzz_type = match self.fuzz_type.to_lowercase().as_str() {
            "string" => FuzzType::String,
            "txt" => FuzzType::Txt,
            "signedint" | "int" => FuzzType::SignedInt,
            "unsignedint" | "uint" => FuzzType::UnsignedInt,
            "jpeg" | "jpg" => FuzzType::Jpeg,
            "png" => FuzzType::Png,
            "pdf" => FuzzType::Pdf,
            _ => return Err(anyhow!("invalid fuzz type")),
        };

        // initialize the rng if a seed is provided otherwise generate one from getrandom
        let rng = if let Some(seed) = self.seed {
            SmallRng::seed_from_u64(seed)
        } else {
            SmallRng::from_os_rng()
        };

        // parse the args and format them as a vector
        let bin_args: Vec<String> = self.bin_args.split(' ').map(String::from).collect();

        let crash_stats = CrashStats {
            total: 0,
            timeout: 0,
            sigill: 0,
            sigabrt: 0,
            sigfpe: 0,
            sigsegv: 0,
            sigpipe: 0,
            sigterm: 0,
        };

        let temp_dir = tempdir().map_err(|_| anyhow!("can't create tempdir"))?;
        // let mutations_dir =
        //     tempdir_in(&temp_dir).map_err(|_| "can't create temporary directory for mutations")?;
        // let corpus_dir =
        //     tempdir_in(temp_dir).map_err(|_| "can't create temporary directory for corpus")?;

        Ok(Config {
            bin_args: bin_args.clone(),
            bin_path: self.bin_path.clone(),
            crash_stats,
            iterations: self.max_iterations,
            report_path: self.report_path.clone(),
            rng,
            temp_dir,
            timeout: self.timeout,
            validated_fuzz_type: validated_fuzz_type.clone(),
        })
    }
}
