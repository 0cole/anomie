use anyhow::Result;
use log::debug;
use std::{
    io::Read,
    os::unix::process::ExitStatusExt,
    process::{Child, Command, Stdio},
    time::Duration,
};
use wait_timeout::ChildExt;

use crate::{errors::ExitStatus, types::Config};

fn run_child(child: &mut Child, timeout: Duration) -> Result<ExitStatus> {
    if let Some(status) = child.wait_timeout(timeout)? {
        let mut stdout = String::new();
        child
            .stdout
            .as_mut()
            .unwrap()
            .read_to_string(&mut stdout)
            .unwrap();
        let mut stderr = String::new();
        child
            .stderr
            .as_mut()
            .unwrap()
            .read_to_string(&mut stderr)
            .unwrap();

        debug!(
            "Code: {:?} (SIG {:?})\nSTDOUT returned: {:?}\nSTDERR returned: {:?}",
            status.code(),
            status.signal().unwrap_or(0),
            stdout,
            stderr,
        );

        if let Some(sig) = status.signal() {
            Ok(ExitStatus::Signal(sig))
        } else if let Some(code) = status.code() {
            Ok(ExitStatus::ExitCode(code))
        } else {
            Ok(ExitStatus::Error("Unknown termination".into()))
        }
    } else {
        child.kill()?;
        child.wait()?;
        Ok(ExitStatus::Timeout(timeout.as_millis()))
    }
}

pub fn run_target_file(config: &Config, binary_args: &[String]) -> Result<ExitStatus> {
    let coalesced_args = binary_args.join(" ");
    debug!(
        "Running: {coalesced_args} on {}",
        config.run_details.bin_path
    );

    let timeout = Duration::from_millis(config.run_details.timeout);
    let mut child = Command::new(&config.run_details.bin_path)
        .args(binary_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let exit_status = run_child(&mut child, timeout)?;
    Ok(exit_status)
}

pub fn run_target_string(config: &Config, fuzz_input: &[u8]) -> Result<ExitStatus> {
    let mut input_args = config.run_details.bin_args.clone();
    let fuzz_string_delim: &[String] = &fuzz_input
        .split(|&b| b == b' ') // use a space to delimit the args
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect::<Vec<String>>();
    input_args.extend_from_slice(fuzz_string_delim);

    let coalesced_args = input_args.join(" ");
    // Print the full command being executed for debug
    if coalesced_args.len() > 300 {
        let slice_position = coalesced_args.char_indices().nth(40).unwrap().0;
        debug!("Running: {}", &coalesced_args[..slice_position]);
    } else {
        debug!("Running: {coalesced_args}");
    }

    let timeout = Duration::from_millis(config.run_details.timeout);
    let mut child = Command::new(&config.run_details.bin_path)
        .args(input_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let exit_status = run_child(&mut child, timeout)?;
    Ok(exit_status)
}
