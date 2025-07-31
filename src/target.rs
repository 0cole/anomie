use crate::errors::ExitStatus;
use log::debug;
use std::{
    os::unix::process::ExitStatusExt,
    process::{Command, Stdio},
};

pub fn run_target(path: &str, input: &[u8], _timeout_ms: u64) -> std::io::Result<ExitStatus> {
    let input_args = input
        .split(|&b| b == b' ') // use a space to delimit the args
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect::<Vec<String>>();

    let coalesced_args = input_args.join(" ");
    // Print the full command being executed for debug
    if coalesced_args.len() > 300 {
        let slice_position = coalesced_args.char_indices().nth(40).unwrap().0;
        debug!("Running: {}", &coalesced_args[..slice_position]);
    } else {
        debug!("Running: {coalesced_args}");
    }

    let output = Command::new(path)
        .args(input_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let status = output.status;

    debug!(
        "Code: {:?} (SIG {:?})\nSTDOUT returned: {:?}\nSTDERR returned: {:?}",
        status.code(),
        status.signal().unwrap_or(0),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let exit_status = if let Some(sig) = status.signal() {
        ExitStatus::Signal(sig)
    } else if let Some(code) = status.code() {
        ExitStatus::ExitCode(code)
    } else {
        ExitStatus::Error("Unknown termination".into())
    };

    Ok(exit_status)
}
