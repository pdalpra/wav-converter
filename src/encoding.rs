use crate::tagging;

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::*;

#[derive(Debug)]
pub struct Job {
    source_file: PathBuf,
    target_file: PathBuf,
}

impl Job {
    pub fn new(source_file: PathBuf, target_file: PathBuf) -> Self {
        Job {
            source_file,
            target_file,
        }
    }

    pub fn convert_to_flac(&self, debug: bool, compression: u8) -> Result<()> {
        if let Some(parent) = self.target_file.parent() {
            fs::create_dir_all(parent)?;
        }
        self.encode(debug, compression)?;
        tagging::tag_file(&self.target_file)?;

        Ok(())
    }

    fn encode(&self, debug: bool, compression: u8) -> Result<()> {
        let mut cmd = Command::new("ffmpeg");

        if !debug {
            cmd.stdout(Stdio::null()).stderr(Stdio::null());
        }

        cmd.arg("-i")
            .arg(&self.source_file)
            .args(&["-map_metadata", "-1"])
            .args(&["-c:a", "flac"])
            .args(&["-compression_level", &compression.to_string()])
            .arg(&self.target_file);

        let status_code = &cmd
            .status()
            .map_err(|err| anyhow!("Error while running ffmpeg: {:?}", err))?;

        if status_code.success() {
            Ok(())
        } else {
            Err(anyhow!(
                "Error while running ffmpeg: exited with status code {}",
                status_code.code().unwrap()
            ))
        }
    }
}
