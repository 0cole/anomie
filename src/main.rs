use std::path::PathBuf;

use analysis::CrashAnalyzer;
use anyhow::Result;
use clap::Parser;
use engine::run_engine_for;
use formats::{jpeg::Jpeg, png::Png, string::FuzzString, txt::Txt};
use log::{error, info};
use types::{Config, FuzzType};

mod analysis;
mod config;
mod engine;
mod errors;
mod formats;
mod mutate;
mod mutations;
mod target;
mod types;
mod utils;

fn main() {
    env_logger::init();

    if let Err(e) = run() {
        error!("{e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let mut config = config::RawConfig::parse().validate()?;
    info!("Parsed config successfully");
    let mut analyzer = analysis::CrashAnalyzer::new(PathBuf::from(&config.report_path));

    utils::initialize(&mut config)?;
    run_engine(&mut analyzer, &mut config)?;
    utils::create_run_json(&analyzer, &config)?;
    utils::print_report(&analyzer, &config)?;

    Ok(())
}

/// if adding a new format type, extend this
fn run_engine(analyzer: &mut CrashAnalyzer, config: &mut Config) -> Result<()> {
    match config.validated_fuzz_type {
        FuzzType::Jpeg => run_engine_for::<Jpeg>(analyzer, config),
        FuzzType::Png => run_engine_for::<Png>(analyzer, config),
        FuzzType::String => run_engine_for::<FuzzString>(analyzer, config),
        FuzzType::Txt => run_engine_for::<Txt>(analyzer, config),
        _ => unreachable!(),
    }
}
