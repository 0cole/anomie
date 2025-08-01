use crate::config::{self, Config};
use crate::fuzzers;
use log::error;

pub fn run(config: &Config) {
    match config.validated_fuzz_type {
        config::Type::String => fuzzers::string::fuzz_string(config),
        config::Type::Txt => fuzzers::txt::fuzz_txt(config),
        _ => error!("Unsupported fuzzing type"),
    }
}
