use anyhow::Result;
use clap::Parser;
use log::{error, info};

mod analysis;
mod config;
mod engine;
mod errors;
mod fuzzers;
mod mutate;
mod target;
mod types;
mod utils;

fn main() -> Result<()> {
    env_logger::init();

    // precedence

    // error!("Error");
    // warn!("Warn");
    // info!("Info");
    // debug!("Debug");
    // trace!("Trace");

    match config::RawConfig::parse().validate() {
        Ok(mut config) => {
            info!("Parsed config successfully");
            utils::create_report_dir(&mut config)?;
            engine::run(&mut config)?;
        }
        Err(err) => {
            error!("Error when parsing config: {err}");
        }
    }

    Ok(())
}
