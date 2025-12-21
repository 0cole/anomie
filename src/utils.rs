use anyhow::Result;
use log::{debug, info};
use std::{
    fs::{self, DirEntry, create_dir},
    path::Path,
};

use crate::types::Config;

pub fn initialize(config: &mut Config) -> Result<()> {
    // create the temporary directories that will be dropped when the fuzzer finishes
    let corpus_dir = config.temp_dir.path().join("corpus/");
    let mutations_dir = config.temp_dir.path().join("mutations/");
    let scratch_dir = config.temp_dir.path().join("scratch/");
    create_dir(corpus_dir)?;
    create_dir(mutations_dir)?;
    create_dir(scratch_dir)?;

    // create the report dir
    if !Path::new(&config.report_path).exists() {
        fs::create_dir(&config.report_path)?;
        debug!("Created report dir at {}", config.report_path);
    }

    // create the next numbered report subdir
    let mut max_index = 0;
    for entry in fs::read_dir(&config.report_path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            if let Some(name) = entry.file_name().to_str() {
                if let Ok(num) = name.parse::<u32>() {
                    if num > max_index {
                        max_index = num;
                    }
                }
            }
        }
    }

    let next_index = max_index + 1;
    let dir_num = format!("{next_index:04}"); // XXXX
    let new_dir_path = config.report_path.clone() + "/" + &dir_num;
    debug!("Creating subdir at {new_dir_path}");
    fs::create_dir(new_dir_path.clone())?;
    fs::create_dir(new_dir_path.clone() + "/SIGILL")?;
    fs::create_dir(new_dir_path.clone() + "/SIGABRT")?;
    fs::create_dir(new_dir_path.clone() + "/SIGFPE")?;
    fs::create_dir(new_dir_path.clone() + "/SIGSEGV")?;
    fs::create_dir(new_dir_path.clone() + "/SIGPIPE")?;
    fs::create_dir(new_dir_path.clone() + "/SIGTERM")?;
    fs::create_dir(new_dir_path.clone() + "/TIMEOUT")?;

    // update the subdir num in config
    info!(
        "Any crashes will be recorded and stored in {:?}",
        &new_dir_path
    );
    config.report_path = new_dir_path;

    Ok(())
}

pub fn filename_bytes(entry: &DirEntry) -> Vec<u8> {
    // weird behavior
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        entry.file_name().as_os_str().as_bytes().to_vec()
    }

    #[cfg(not(unix))]
    {
        entry
            .file_name()
            .to_string_lossy()
            .into_owned()
            .into_bytes()
    }
}

pub fn create_report_json(config: &Config) -> Result<()> {
    let report_json = serde_json::to_string(&config)?;
    fs::write(config.report_path.clone() + "/report.json", report_json)?;
    Ok(())
}
