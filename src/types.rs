// Describes the input type that caused the crash. Needed for the
// analysis section where I save the specific string/file that
// caused a crash.
pub enum StructuredInput {
    StringInput(Vec<u8>),      // Contains an array of bytes that caused the crash
    FileInput(String, String), // Contains the file path and the extension used
}
