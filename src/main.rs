use std::io;

use clap::Parser;
use utils::create_report_dir;

mod config;
mod engine;
mod fuzzers;
mod mutate;
mod target;
mod utils;

fn main() -> io::Result<()> {
    let config = config::RawConfig::parse().validate().unwrap(); // TODO better error handling
    create_report_dir(&config)?;
    engine::run(config);
    Ok(())
}
