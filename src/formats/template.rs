use anyhow::Result;
use rand::rngs::SmallRng;

pub trait FileFormat {
    type Model;
    const EXT: &'static str;

    // parse an array of bytes into a file
    fn parse(input: &[u8]) -> Result<Self::Model>;

    // generate a vector of bytes from a model
    fn generate(model: Self::Model) -> Result<Vec<u8>>;

    // generate a corpus
    fn generate_corpus(rng: &mut SmallRng, corpus_dir: &str) -> Result<()>;

    // apply mutations to a model
    fn mutate(rng: &mut SmallRng, model: &mut Self::Model) -> Result<String>;

    // saves the file, returns the path to the saved file as a string
    // fn save_file(model: Self::Model) -> Result<String>;
}
