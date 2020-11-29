use crate::format::Format;
use crate::tagging;

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::*;

#[derive(Debug)]
pub struct Job {
    source_file: PathBuf,
    target_file: PathBuf,
    format: Format,
    compression: u8,
}

impl Job {
    pub fn new(source_file: PathBuf, target_file: PathBuf, format: Format, compression: u8) -> Self {
        Job {
            source_file,
            target_file,
            format,
            compression,
        }
    }

    pub fn convert(&self, debug: bool) -> Result<()> {
        if let Some(parent) = self.target_file.parent() {
            fs::create_dir_all(parent)?;
        }
        self.encode(debug)?;
        tagging::tag_file(&self.target_file)?;

        Ok(())
    }

    fn encode(&self, debug: bool) -> Result<()> {
        let mut cmd = Command::new("ffmpeg");

        if !debug {
            cmd.stdout(Stdio::null()).stderr(Stdio::null());
        }

        cmd.arg("-i")
            .arg(&self.source_file)
            .args(&["-map_metadata", "-1"])
            .args(&["-c:a", &self.format.codec_name()])
            .args(self.format_specific_options())
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

    fn format_specific_options(&self) -> Vec<String> {
        match self.format {
            Format::Flac => vec!["-compression_level".to_string(), self.compression.to_string()],
            Format::Alac => vec![],
        }
    }
}
