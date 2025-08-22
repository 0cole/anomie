use rand::rngs::SmallRng;

// Fuzzer global config, this is the struct used after input validation
// (eg. verifying the binary exists, making sure the FuzzType is valid)
#[derive(Debug)]
pub struct Config {
    pub bin_args: Vec<String>,
    pub bin_path: String,
    pub max_iterations: usize,
    pub report_path: String,
    pub rng: SmallRng,
    // pub timeout: u64,
    pub validated_fuzz_type: FuzzType,
}

// Describes the types supported by the fuzzer
#[derive(Debug, Clone)]
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
