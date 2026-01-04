use std::fs::{self, File, read_dir};
use std::io::Write;
use std::marker::PhantomData;

use anyhow::Result;
use log::{debug, info};
use rand::Rng;

use crate::analysis::CrashAnalyzer;
use crate::errors::ExitStatus;
use crate::formats::template::FileFormat;
use crate::target::{run_target_file, run_target_string};
use crate::types::{Config, FuzzType, StructuredInput};
use crate::utils::filename_bytes;

pub struct Engine<'a, F: FileFormat> {
    analyzer: &'a mut CrashAnalyzer,
    config: &'a mut Config,
    _marker: PhantomData<F>,
}

pub fn run_engine_for<T: FileFormat>(
    analyzer: &mut CrashAnalyzer,
    config: &mut Config,
) -> Result<()> {
    let mut engine = Engine::<T>::new(analyzer, config);
    engine.run()
}

impl<'a, F: FileFormat> Engine<'a, F> {
    pub fn new(analyzer: &'a mut CrashAnalyzer, config: &'a mut Config) -> Self {
        Self {
            analyzer,
            config,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        info!("Beginning fuzzing...");

        let corpus_dir = self.config.temp_dir.path().join("corpus");
        let mutations_dir = self.config.temp_dir.path().join("mutations");
        F::generate_corpus(&mut self.config.rng, &corpus_dir)?;

        let corpus: Vec<_> = read_dir(&corpus_dir)?.filter_map(Result::ok).collect();
        let corpus_size = corpus.len();

        for i in 0..self.config.iterations {
            let rand_idx = self.config.rng.random_range(0..corpus_size);
            let random_file = &corpus[rand_idx];

            let content: &[u8] = match self.config.validated_fuzz_type {
                FuzzType::String => &filename_bytes(random_file),
                FuzzType::Txt | FuzzType::Jpeg | FuzzType::Png => &fs::read(random_file.path())?,
                _ => unreachable!(),
            };

            // mutate input
            let mut mutation_array: Vec<String> = Vec::new();
            let mutation_count = self.config.rng.random_range(0..5);
            let mut model: F::Model = F::parse(content)?;
            for _ in 0..mutation_count {
                let mutation_string = F::mutate(&mut self.config.rng, &mut model)?;
                debug!("{mutation_string}");
                mutation_array.push(mutation_string);
            }

            let mutated_bytes = F::generate(model)?;

            let (structured_input, result) = match self.config.validated_fuzz_type {
                FuzzType::Txt | FuzzType::Jpeg | FuzzType::Png => {
                    let mutated_file_name = format!("{i}.{}", F::EXT);
                    let mut mutated_file = File::create(mutations_dir.join(&mutated_file_name))?;
                    mutated_file.write_all(&mutated_bytes)?;
                    (
                        StructuredInput::FileInput {
                            path: mutations_dir.join(&mutated_file_name),
                            extension: F::EXT.to_string(),
                        },
                        run_target_file(self.config, mutated_file_name.as_str())
                            .unwrap_or(ExitStatus::ExitCode(0)),
                    )
                }
                // unique handling for fuzzing the filename itself
                FuzzType::String => (
                    StructuredInput::StringInput(mutated_bytes.clone()),
                    run_target_string(self.config, &mutated_bytes)
                        .unwrap_or(ExitStatus::ExitCode(0)),
                ),
                _ => unreachable!(),
            };

            self.analyzer
                .analyze(i, result, structured_input, mutation_array)?;
        }
        Ok(())
    }
}
