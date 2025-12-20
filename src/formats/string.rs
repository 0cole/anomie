use std::{fs, io::Write};

use anyhow::Result;
use rand::rngs::SmallRng;

use crate::mutate::mutate_bytes;

use super::template::FileFormat;

pub struct FuzzString;
pub struct FuzzStringModel {
    pub filename: Vec<u8>,
}

impl FileFormat for FuzzString {
    type Model = FuzzStringModel;
    const EXT: &str = "";

    fn parse(input: &[u8]) -> Result<Self::Model> {
        Ok(FuzzStringModel {
            filename: input.to_vec(),
        })
    }

    fn generate(model: Self::Model) -> Result<Vec<u8>> {
        Ok(model.filename)
    }

    fn generate_corpus(rng: &mut rand::prelude::SmallRng, corpus_dir: &str) -> Result<()> {
        let filenames = vec![
            "My name is Cole.".to_string(),
            String::new(),
            "\n".to_string(),
            "\0hello".to_string(),
            "\'".to_string(),
            "My name is Cole".repeat(10),
            String::from_utf8_lossy(b"\xFF\xFF").to_string(),
            String::from_utf8_lossy(b"\x00\x00\x00").to_string(),
        ];

        for filename in filenames {
            let filepath = format!("{corpus_dir}/{filename}.txt");
            let mut file = fs::File::create(&filepath)?;
            file.write_all(b"\x12")?;
        }

        Ok(())
    }

    fn mutate(rng: &mut SmallRng, model: &mut Self::Model) -> Result<String> {
        Ok(mutate_bytes(rng, &mut model.filename))
    }
}
