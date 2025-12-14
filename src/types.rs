use rand::rngs::SmallRng;
use serde::{Deserialize, Serialize};

// Fuzzer global config, this is the struct used after input validation
// (eg. verifying the binary exists, making sure the FuzzType is valid)
#[derive(Debug)]
pub struct Config {
    pub report_path: String,
    pub rng: SmallRng,
    pub run_details: RunDetails,
}

// The data that should be serialized into a report.json. This allows
// for reproducibility and details for each report.
#[derive(Serialize, Deserialize, Debug)]
pub struct RunDetails {
    pub bin_args: Vec<String>,
    pub bin_path: String,
    pub max_iterations: usize,
    pub validated_fuzz_type: FuzzType,
    pub timeout: u64,
    pub total_hits: u64,
    pub sigill_hits: u64,
    pub sigabrt_hits: u64,
    pub sigfpe_hits: u64,
    pub sigsegv_hits: u64,
    pub sigpipe_hits: u64,
    pub sigterm_hits: u64,
    pub timeout_hits: u64,
}

// Describes the types supported by the fuzzer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FuzzType {
    String,
    Txt,
    Jpeg,        // TODO
    Png,         // TODO
    Pdf,         // TODO
    SignedInt,   // not currently implemented
    UnsignedInt, // not currently implemented
}

// Describes the input type that caused the crash. Needed for the
// analysis section where I save the specific string/file that
// caused a crash.
pub enum StructuredInput {
    StringInput(Vec<u8>),      // Contains an array of bytes that caused the crash
    FileInput(String, String), // Contains the file path and the extension used
}
