use std::process::{Command, Stdio};

pub fn run_target(path: &str, input: &[u8], _timeout_ms: u64, debug: bool) -> std::io::Result<i32> {
    let input_args = input
        .split(|&b| b == b' ') // use a space to delimit the args
        .map(|s| String::from_utf8_lossy(s).into_owned())
        .collect::<Vec<String>>();

    let coalesced_args = input_args.join(" ");
    // Print the full command being executed for debug
    if debug {
        if coalesced_args.len() > 300 {
            println!("Running: {}", coalesced_args.to_string().split_at(40).0);
        } else {
            println!("Running: {coalesced_args}");
        }
    }

    let output = Command::new(path)
        .args(input_args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output()?;

    if debug {
        println!(
            "STDOUT returned: {}",
            String::from_utf8_lossy(&output.stdout)
        );
    }
    Ok(output.status.code().unwrap_or(-1))
}
