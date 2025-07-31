#[derive(Debug, PartialEq, Eq)]
pub enum ExitStatus {
    ExitCode(i32),
    Signal(i32),
    Error(String),
}

pub const SIGILL: i32 = 4; // abnormal termination
pub const SIGABRT: i32 = 6; // abnormal termination
pub const SIGFPE: i32 = 8; // floating point exception
pub const SIGSEGV: i32 = 11; // seg fault
pub const SIGPIPE: i32 = 13; // pipe error
pub const SIGTERM: i32 = 15; // termination signal
