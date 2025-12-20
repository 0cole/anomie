use anyhow::Result;
use log::{debug, info};
use std::{
    fs::{self, DirEntry},
    path::{Path, PathBuf},
};

use crate::types::{Config, FuzzType};

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
    fs::create_dir(new_dir_path.clone() + "/TIMEOUT")?;

    // update the subdir num in config
    info!(
        "Any crashes will be recorded and stored in {:?}",
        &new_dir_path
    );
    config.report_path = new_dir_path;

    Ok(())
}

pub fn initialize_dirs(fuzz_type: &FuzzType) -> Result<()> {
    let extension = match fuzz_type {
        FuzzType::Txt => "txt",
        FuzzType::Jpeg => "jpg",
        FuzzType::Png => "png",
        FuzzType::Pdf => "pdf",
        FuzzType::String | FuzzType::SignedInt | FuzzType::UnsignedInt => "",
    };

    let temp_dir = "temp";
    if Path::new(temp_dir).exists() {
        fs::remove_dir_all(temp_dir)?;
    }
    fs::create_dir(temp_dir)?;

    fs::create_dir_all("corpus")?;
    if !extension.is_empty() {
        let corpus_extension_dir = format!("corpus/{extension}");
        fs::create_dir_all(&corpus_extension_dir)?;
    }

    Ok(())
}

pub fn filename_bytes(entry: &DirEntry) -> Vec<u8> {
    // weird behavior
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        return entry.file_name().as_os_str().as_bytes().to_vec();
    }

    #[cfg(not(unix))]
    {
        return entry
            .file_name()
            .to_string_lossy()
            .into_owned()
            .into_bytes();
    }
}

pub fn create_report_json(config: &Config) -> Result<()> {
    let report_json = serde_json::to_string(&config)?;
    fs::write(config.report_path.clone() + "/report.json", report_json)?;
    Ok(())
}

pub fn clean_up(fuzz_type: &FuzzType) -> Result<()> {
    let extension = match fuzz_type {
        FuzzType::Txt => "txt",
        FuzzType::Jpeg => "jpg",
        FuzzType::Png => "png",
        FuzzType::Pdf => "pdf",
        FuzzType::String | FuzzType::SignedInt | FuzzType::UnsignedInt => "",
    };

    let temp_dir = "temp";

    if !extension.is_empty() {
        let files: Vec<PathBuf> = fs::read_dir(format!("corpus/{extension}"))
            .unwrap()
            .filter_map(|entry| Some(entry.ok()?.path()))
            .collect();
        for file in files {
            fs::remove_file(file)?;
        }
    }

    fs::remove_dir_all(temp_dir)?;
    Ok(())
}
