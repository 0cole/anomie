use clap::Parser;

mod config;
mod engine;
mod fuzzers;
mod mutate;
mod target;

fn main() {
    let config = config::RawConfig::parse().validate().unwrap(); // TODO better error handling
    engine::run(config);
}
