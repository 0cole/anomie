use rand::random_range;

use crate::{mutate, target};

fn basic_corpus() -> Vec<String> {
    vec![
        "My name is Cole.".to_string(),
        String::new(),
        "\n".to_string(),
        "\0hello".to_string(),
        "\'".to_string(),
        "My name is Cole".repeat(1000),
        String::from_utf8_lossy(b"\xFF\xFF").to_string(),
        String::from_utf8_lossy(b"\x00\x00\x00").to_string(),
    ]
}

pub fn fuzz_string(config: &crate::config::Config) {
    let corpus = basic_corpus();
    let mut input: String;
    for _ in 0..config.max_iterations {
        input = corpus[random_range(..corpus.len())].clone();
        let mutated = mutate::mutate_string(&input);
        let result = target::run_target(
            &config.bin_path,
            mutated.as_bytes(),
            config.timeout,
            config.debug,
        );
        match result {
            Ok(code) if code != 0 => {
                println!("Interesting input: {mutated}");
                println!("{code}");
            }
            _ => {
                // println!("{}", result.unwrap());
            }
        }
        // input = mutated;
    }
}
