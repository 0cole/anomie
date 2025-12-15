use rand::rngs::SmallRng;
use serde::Serialize;

// Fuzzer global config, this is the struct used after input validation
// (eg. verifying the binary exists, making sure the FuzzType is valid)
// at the end of a run, the config is serialized to a json which is
// included in the run report. The smallrng is excluded. TODO: add seed
#[derive(Serialize, Debug)]
pub struct Config {
    pub bin_args: Vec<String>,
    pub bin_path: String,
    pub crash_stats: CrashStats,
    pub iterations: usize,
    pub report_path: String,

    #[serde(skip)]
    pub rng: SmallRng, // skip this when serializing

    pub timeout: u64,
    pub validated_fuzz_type: FuzzType,
}

// Struct containing all of the possible crashes, when a crash occurs,
// the value here is incremented by 1.
#[derive(Serialize, Debug)]
pub struct CrashStats {
    pub total: u64,
    pub sigill: u64,
    pub sigabrt: u64,
    pub sigfpe: u64,
    pub sigsegv: u64,
    pub sigpipe: u64,
    pub sigterm: u64,
    pub timeout: u64,
}

// Describes the types supported by the fuzzer
#[derive(Debug, Clone, Serialize)]
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
