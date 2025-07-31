use log::error;

use crate::config::Config;
use crate::fuzzers;

pub fn run(config: &Config) {
    match config.validated_fuzz_type {
        crate::config::Type::String => fuzzers::string::fuzz_string(&config),
        _ => error!("Unsupported fuzzing type"),
    }
}

// TODO maybe should use a generic here rather than string
pub fn record_behavior(input: String, config: &Config) {}
