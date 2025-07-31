use crate::{analysis::analyze_result, config, errors::ExitStatus, mutate, target};
use rand::random_range;

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

pub fn fuzz_string(config: &config::Config) {
    let corpus = basic_corpus();
    let mut input: String;
    for id in 0..config.max_iterations {
        input = corpus[random_range(..corpus.len())].clone();
        let mutated = mutate::mutate_string(&input);
        let result = target::run_target(&config.bin_path, mutated.as_bytes(), config.timeout)
            .unwrap_or(ExitStatus::ExitCode(0));
        analyze_result(&config.report_path, result, id, mutated.as_bytes());
        // input = mutated;
    }
}
