use crate::flags::EncodingOptions;
use crate::format::Format;
use crate::tagging;

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use anyhow::*;

pub struct FileToConvert {
    source_file: PathBuf,
    target_file: PathBuf,
}

impl FileToConvert {
    pub fn new(source_file: PathBuf, target_file: PathBuf) -> Self {
        FileToConvert {
            source_file,
            target_file,
        }
    }

    pub fn convert(&self, encoding_options: &EncodingOptions, debug: bool) -> Result<()> {
        if let Some(parent) = self.target_file.parent() {
            fs::create_dir_all(parent)?;
        }
        self.encode(&encoding_options, debug)?;
        tagging::tag_file(&self.target_file)?;

        Ok(())
    }

    fn encode(&self, encoding_options: &EncodingOptions, debug: bool) -> Result<()> {
        let mut cmd = Command::new("ffmpeg");

        if !debug {
            cmd.stdout(Stdio::null()).stderr(Stdio::null());
        }

        cmd.arg("-i")
            .arg(&self.source_file)
            .args(&["-map_metadata", "-1"])
            .args(&["-c:a", &encoding_options.format.codec_name()])
            .args(Self::format_specific_options(encoding_options))
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

    fn format_specific_options(encoding_options: &EncodingOptions) -> Vec<String> {
        match encoding_options.format {
            Format::Flac => vec![
                "-compression_level".to_string(),
                encoding_options.compression.to_string(),
            ],
            Format::Alac => vec![],
        }
    }
}
