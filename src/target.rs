use crate::errors::ExitStatus;
use log::debug;
use std::{
    io,
    os::unix::process::ExitStatusExt,
    process::{Command, Output, Stdio},
};

fn assess_output(output: &Output) -> ExitStatus {
    let status = output.status;

    debug!(
        "Code: {:?} (SIG {:?})\nSTDOUT returned: {:?}\nSTDERR returned: {:?}",
        status.code(),
        status.signal().unwrap_or(0),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    if let Some(sig) = status.signal() {
        ExitStatus::Signal(sig)
    } else if let Some(code) = status.code() {
        ExitStatus::ExitCode(code)
    } else {
        ExitStatus::Error("Unknown termination".into())
    }
}

pub fn run_target_file(binary_args: &[String], binary_path: &str) -> io::Result<ExitStatus> {
    let coalesced_args = binary_args.join(" ");
    debug!("Running: {coalesced_args} on {binary_path}");

    let output = Command::new(binary_path)
        .args(binary_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let exit_status = assess_output(&output);
    Ok(exit_status)
}

pub fn run_target_string(path: &str, input: &[u8], _timeout_ms: u64) -> io::Result<ExitStatus> {
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

    let exit_status = assess_output(&output);
    Ok(exit_status)
}
