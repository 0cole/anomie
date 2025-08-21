use crate::fuzzers;
use crate::types::{Config, FuzzType};
use log::error;

pub fn run(config: &mut Config) {
    match config.validated_fuzz_type {
        FuzzType::String => fuzzers::string::fuzz_string(config),
        FuzzType::Txt => fuzzers::txt::fuzz_txt(config),
        FuzzType::Jpeg => fuzzers::jpeg::fuzz_jpeg(config),
        _ => error!("Unsupported fuzzing type"),
    }
}
