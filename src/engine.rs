use crate::fuzzers;
use crate::types::{Config, FuzzType};
use log::error;

pub fn run(config: &Config) {
    match config.validated_fuzz_type {
        FuzzType::String => fuzzers::string::fuzz_string(config),
        FuzzType::Txt => fuzzers::txt::fuzz_txt(config),
        _ => error!("Unsupported fuzzing type"),
    }
}
