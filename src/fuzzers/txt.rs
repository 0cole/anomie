use anyhow::Result;
use log::info;
use rand::{Rng, rngs::SmallRng};
use std::{fs, io::Write};

use crate::{
    analysis::analyze_result,
    errors::ExitStatus,
    mutate::mutate_bytes,
    target::run_target_file,
    types::{Config, StructuredInput},
};

const CORPUS_SIZE: usize = 20;

fn generate_txt_corpus(rng: &mut SmallRng, corpus_dir: &str) -> Result<()> {
    // generate CORPUS_SIZE random txt files
    for i in 0..CORPUS_SIZE {
        let mut content = Vec::new();
        let length = rng.random_range(0..1000);
        for _ in 0..length {
            content.push(rng.random::<u8>());
        }

        let filename = format!("{corpus_dir}/{i}.txt");
        let mut file = fs::File::create(&filename)?;
        file.write_all(&content)?;
    }

    info!("Generated corpus files in {corpus_dir}");
    Ok(())
}

pub fn fuzz_txt(config: &mut Config) -> Result<()> {
    info!("Beginning txt fuzzing");

    // create the corpus dir to store our basic txt files
    let corpus_txt_dir = "corpus/txt";
    generate_txt_corpus(&mut config.rng, corpus_txt_dir)?;
    fs::create_dir_all(corpus_txt_dir)?;

    // setup the args
    let mutated_file_path = "temp/mutated.txt";
    let mut bin_args_plus_file = config.run_details.bin_args.clone();
    bin_args_plus_file.push(mutated_file_path.to_string());

    for id in 0..config.run_details.max_iterations {
        // pick a random file from our corpus
        let file_num = config.rng.random_range(0..CORPUS_SIZE);
        let file = format!("{corpus_txt_dir}/{file_num}.txt");

        // apply mutations to file
        let mut bytes = fs::read(file)?;
        mutate_bytes(&mut bytes);
        fs::write(mutated_file_path, &bytes)?;

        // TODO add arg functionality
        let result =
            run_target_file(config, &bin_args_plus_file).unwrap_or(ExitStatus::ExitCode(0));
        let structured_input =
            StructuredInput::FileInput(mutated_file_path.to_string(), "txt".to_string());
        analyze_result(
            &config.report_path,
            &mut config.run_details,
            result,
            id,
            structured_input,
        )?;
    }

    Ok(())
}
