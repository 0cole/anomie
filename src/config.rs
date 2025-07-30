use clap::Parser;
use std::fs;

#[derive(Parser, Debug)]
pub struct RawConfig {
    #[arg(short, long)]
    pub bin_path: String,

    #[arg(short, long, action)]
    // print the mutated object and stdout for each fuzz attempt
    pub debug: bool,

    #[arg(long, default_value = "string")]
    pub fuzz_type: String,

    #[arg(long, default_value_t = 1000)]
    pub max_iterations: usize,

    #[arg(long, default_value_t = 100)]
    pub timeout: u64,

    #[arg(short, long, default_value = "./reports")]
    pub report_path: String,
}

#[derive(Debug, Clone)]
pub enum Type {
    String,
    SignedInt,
    UnsignedInt,
}

#[derive(Debug)]
pub struct Config {
    pub bin_path: String,
    pub debug: bool,
    pub max_iterations: usize,
    pub report_path: String,
    pub timeout: u64,
    pub validated_fuzz_type: Type,
}

impl RawConfig {
    pub fn validate(&self) -> Result<Config, &'static str> {
        // validate the binary passed in
        let metadata = fs::metadata(&self.bin_path).map_err(
            |_| "invalid binary path, make sure the binary is in the root dir of anomie",
        )?;
        if !metadata.is_file() {
            dbg!(&self.bin_path);
            return Err("path does not correspond to an actual binary");
        }

        // validate the type passed in
        let validated_fuzz_type = match self.fuzz_type.to_lowercase().as_str() {
            "string" => Type::String,
            "signedint" | "int" => Type::SignedInt,
            "unsignedint" | "uint" => Type::UnsignedInt,
            _ => return Err("invalid fuzz type"),
        };
        Ok(Config {
            bin_path: self.bin_path.clone(),
            debug: self.debug,
            max_iterations: self.max_iterations,
            report_path: self.report_path.clone(),
            timeout: self.timeout,
            validated_fuzz_type,
        })
    }
}
