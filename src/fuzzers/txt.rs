use crate::{config, mutate::mutate_bytes};
use log::info;
use rand::Rng;
use std::{fs, io::Write};

const CORPUS_SIZE: usize = 20;

fn generate_txt_corpus(corpus_dir: &str) {
    let mut rng = rand::rng();

    // generate CORPUS_SIZE random txt files
    for i in 0..CORPUS_SIZE {
        let mut content = Vec::new();
        let length = rng.random_range(0..1000);
        for _ in 0..length {
            content.push(rng.random::<u8>());
        }

        let filename = format!("{corpus_dir}/{i}.txt");
        let mut file = fs::File::create(&filename).unwrap();
        file.write_all(&content).unwrap();
    }

    info!("Generated corpus files in {corpus_dir}");
}

pub fn fuzz_txt(config: &config::Config) {
    let mut rng = rand::rng();

    // create the corpus dir to store our basic txt files
    let corpus_txt_dir = "corpus/txt";
    generate_txt_corpus(corpus_txt_dir);
    fs::create_dir_all(corpus_txt_dir).unwrap();

    for id in 0..config.max_iterations {
        // pick a random file from our corpus
        let file_num = rng.random_range(0..CORPUS_SIZE);
        let file = format!("{corpus_txt_dir}/{file_num}.txt");

        // apply mutations to file
        let mut bytes = fs::read(file).unwrap();
        mutate_bytes(&mut bytes);

        // write new mutated file
        fs::write("mutated.txt", &bytes).unwrap();
    }
}
