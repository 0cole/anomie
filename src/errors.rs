#[derive(Debug, PartialEq, Eq)]
pub enum ExitStatus {
    ExitCode(i32),
    Signal(i32),
    Error(String),
}

pub const SIGSEGV: i32 = 11;
