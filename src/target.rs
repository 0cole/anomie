use std::process::{Command, Stdio};

use log::debug;

pub fn run_target(path: &str, input: &[u8], _timeout_ms: u64) -> std::io::Result<i32> {
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

    debug!(
        "STATUS: {:?}\nSTDOUT returned: {:?}\nSTDERR returned: {:?}",
        &output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    Ok(output.status.code().unwrap_or(<i32>::MIN))
}
