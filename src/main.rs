use clap::Parser;
use log::error;
use std::io;
use utils::create_report_dir;

mod config;
mod engine;
mod errors;
mod fuzzers;
mod mutate;
mod target;
mod utils;

fn main() -> io::Result<()> {
    env_logger::init();

    // precedence

    // error!("Error");
    // warn!("Warn");
    // info!("Info");
    // debug!("Debug");
    // trace!("Trace");

    match config::RawConfig::parse().validate() {
        Ok(config) => {
            create_report_dir(&config)?;
            engine::run(&config);
        }
        Err(err) => {
            error!("{err}");
        }
    }

    Ok(())
}
