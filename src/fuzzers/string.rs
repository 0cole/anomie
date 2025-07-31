use log::{debug, info};
use rand::random_range;

use crate::{
    config,
    errors::{self, ExitStatus},
    mutate,
    target::{self},
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

pub fn fuzz_string(config: &config::Config) {
    let corpus = basic_corpus();
    let mut input: String;
    for _ in 0..config.max_iterations {
        input = corpus[random_range(..corpus.len())].clone();
        let mutated = mutate::mutate_string(&input);
        let result = target::run_target(&config.bin_path, mutated.as_bytes(), config.timeout)
            .unwrap_or(ExitStatus::ExitCode(0));
        match result {
            ExitStatus::ExitCode(code) => {
                debug!("Process exited gracefully with code {code}");
            }
            ExitStatus::Signal(sig) => match sig {
                errors::SIGILL => info!("Hit! Process encountered an illegal instruction"),
                errors::SIGABRT => info!("Hit! Process executed the abort function"),
                errors::SIGFPE => info!("Hit! Process encountered a floating point exception"),
                errors::SIGSEGV => info!("Hit! Process encountered a segmentation fault"),
                errors::SIGPIPE => info!("Hit! Process encountered a pipe error"),
                errors::SIGTERM => info!("Hit! Process was terminated"),
                _ => info!("Hit! Process crashed with unknown signal {sig}"),
            },
            // ExitStatus::Timeout => {
            //     info!("Hit! Process timed out");
            // }
            ExitStatus::Error(msg) => {
                info!("Hit! Process execution error: {msg}");
            }
        }
        // input = mutated;
    }
}
