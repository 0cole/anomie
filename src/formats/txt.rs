use super::template::FileFormat;
use crate::mutate::mutate_bytes;
use anyhow::Result;
use rand::{Rng, rngs::SmallRng};
use std::{fs, io::Write, path::Path};

pub struct Txt;
pub struct TxtModel {
    pub bytes: Vec<u8>,
}

impl FileFormat for Txt {
    type Model = TxtModel;
    const EXT: &str = "txt";

    fn parse(input: &[u8]) -> Result<Self::Model> {
        Ok(TxtModel {
            bytes: input.to_vec(),
        })
    }

    fn generate(model: Self::Model) -> Result<Vec<u8>> {
        Ok(model.bytes)
    }

    fn generate_corpus(rng: &mut SmallRng, corpus_dir: &Path) -> Result<()> {
        // create 20 randomly generated txt files
        for i in 0..20 {
            let mut content = Vec::new();
            let length = rng.random_range(0..1000);
            for _ in 0..length {
                content.push(rng.random::<u8>());
            }

            let file_name = format!("{i}.txt");
            let mut file = fs::File::create(corpus_dir.join(file_name))?;
            file.write_all(&content)?;
        }
        Ok(())
    }

    fn mutate(rng: &mut SmallRng, model: &mut Self::Model) -> Result<String> {
        Ok(mutate_bytes(rng, &mut model.bytes))
    }
}
