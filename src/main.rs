use anyhow::Result;
use clap::Parser;
use engine::Engine;
use formats::{jpeg::Jpeg, string::FuzzString, txt::Txt};
use log::{error, info};
use types::FuzzType;

mod analysis;
mod config;
mod engine;
mod errors;
mod formats;
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
            utils::initialize(&mut config)?;

            match config.validated_fuzz_type {
                FuzzType::Txt => {
                    let mut engine = Engine::<Txt>::new(&mut config);
                    engine.run()?;
                }
                FuzzType::Jpeg => {
                    let mut engine = Engine::<Jpeg>::new(&mut config);
                    engine.run()?;
                }
                FuzzType::String => {
                    let mut engine = Engine::<FuzzString>::new(&mut config);
                    engine.run()?;
                }
                _ => error!("Unsupported fuzz type"),
            }

            utils::create_report_json(&config)?;
            utils::clean_up(&config);
        }
        Err(err) => {
            error!("Error when parsing config: {err}");
        }
    }

    Ok(())
}
