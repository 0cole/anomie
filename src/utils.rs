use std::{fs, io, path::Path};

use crate::config::Config;

pub fn create_report_dir(config: &Config) -> io::Result<()> {
    if Path::new(&config.report_path).exists() {
        return Ok(());
    }

    fs::create_dir(&config.report_path)?;
    if config.debug {
        println!("Created report dir at {}", config.report_path);
    }
    Ok(())
}
