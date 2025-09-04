use anyhow::Result;
use log::info;
use rand::random_range;

use crate::{
    analysis::analyze_result,
    errors::ExitStatus,
    mutate, target,
    types::{Config, StructuredInput},
};

fn basic_corpus() -> Vec<String> {
    vec![
        "My name is Cole.".to_string(),
        String::new(),
        "\n".to_string(),
        "\0hello".to_string(),
        "\'".to_string(),
        "My name is Cole".repeat(10),
        String::from_utf8_lossy(b"\xFF\xFF").to_string(),
        String::from_utf8_lossy(b"\x00\x00\x00").to_string(),
    ]
}

pub fn fuzz_string(config: &Config) -> Result<()> {
    info!("Beginning string fuzzing");

    let corpus = basic_corpus();
    let mut input: String;

    for id in 0..config.max_iterations {
        input = corpus[random_range(..corpus.len())].clone();
        let mutated = mutate::mutate_string(&input).into_bytes();
        let structured_input = StructuredInput::StringInput(mutated.clone());
        let result =
            target::run_target_string(&config, &mutated).unwrap_or(ExitStatus::ExitCode(0));
        analyze_result(&config.report_path, result, id, structured_input)?;
        // input = mutated;
    }

    Ok(())
}
