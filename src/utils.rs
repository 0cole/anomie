use anyhow::Result;
use log::{debug, info};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::types::Config;

pub fn create_report_dir(config: &mut Config) -> Result<()> {
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

    // update the subdir num in config
    info!(
        "Any crashes will be recorded and stored in {:?}",
        &new_dir_path
    );
    config.report_path = new_dir_path;

    Ok(())
}

pub fn clean_up(dir: &str, extension: &str) -> Result<()> {
    let files: Vec<PathBuf> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|entry| Some(entry.ok()?.path()))
        .collect();
    for file in files {
        fs::remove_file(file)?;
    }

    fs::remove_file(format!("mutated.{extension}"))?;
    Ok(())
}
