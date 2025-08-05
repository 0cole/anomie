use clap::Parser;
use std::fs;

#[derive(Parser, Debug)]
pub struct RawConfig {
    #[arg(short, long)]
    pub bin_path: String,

    #[arg(long, default_value = "string")]
    pub fuzz_type: String,

    #[arg(long, default_value_t = 1000)]
    pub max_iterations: usize,

    #[arg(long, default_value_t = 100)]
    pub timeout: u64,

    #[arg(short, long, default_value = "./reports")]
    pub report_path: String,

    // everything after is part of args
    #[arg(last = true)]
    pub bin_args: String,
}

#[derive(Debug, Clone)]
pub enum Type {
    String,
    Txt,
    SignedInt,
    UnsignedInt,
}

#[derive(Debug)]
pub struct Config {
    pub bin_args: Vec<String>,
    pub bin_path: String,
    pub max_iterations: usize,
    pub report_path: String,
    pub timeout: u64,
    pub validated_fuzz_type: Type,
}

impl RawConfig {
    pub fn validate(&self) -> Result<Config, &'static str> {
        // validate the binary passed in
        let metadata = fs::metadata(&self.bin_path)
            .map_err(|_| "invalid binary path, double check the path exists")?;
        if !metadata.is_file() {
            return Err("path does not correspond to a binary");
        }

        // validate the type passed in
        let validated_fuzz_type = match self.fuzz_type.to_lowercase().as_str() {
            "string" => Type::String,
            "txt" => Type::Txt,
            "signedint" | "int" => Type::SignedInt,
            "unsignedint" | "uint" => Type::UnsignedInt,
            _ => return Err("invalid fuzz type"),
        };

        // parse the args and format them as a vector
        let bin_args: Vec<String> = self.bin_args.split(' ').map(String::from).collect();

        Ok(Config {
            bin_args,
            bin_path: self.bin_path.clone(),
            max_iterations: self.max_iterations,
            report_path: self.report_path.clone(),
            timeout: self.timeout,
            validated_fuzz_type,
        })
    }
}
