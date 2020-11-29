use crate::format::Format;

use std::path::PathBuf;

use anyhow::*;
use log::{debug, LevelFilter};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opts {
    /// Silence all output
    #[structopt(short, long)]
    pub quiet: bool,

    /// Enable debug logs
    #[structopt(short, long)]
    pub debug: bool,

    #[structopt(short, long, default_value = "flac")]
    pub format: Format,

    /// Set FLAC compression level
    #[structopt(short, long)]
    pub compression: Option<u8>,

    /// Enable dry-run
    #[structopt(long)]
    pub dry_run: bool,

    /// Input folder containing the WAV files to convert.
    #[structopt(parse(from_os_str))]
    pub src: PathBuf,

    /// Output folder for files converted to FLAC.
    #[structopt(parse(from_os_str))]
    pub dest: PathBuf,
}

impl Opts {
    pub fn validate(self) -> Result<Self> {
        Self::validate_directory(&self.src)?;
        if self.format != Format::Flac {
            if let Some(compression) = &self.compression {
                debug!(
                    "Ignoring compression level ({}): not supported by {}",
                    compression,
                    self.format.codec_name()
                )
            }
        }
        Ok(self)
    }

    pub fn log_level(&self) -> LevelFilter {
        if self.quiet {
            LevelFilter::Off
        } else if self.debug {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        }
    }

    fn validate_directory(directory: &PathBuf) -> Result<()> {
        if directory.is_dir() {
            Ok(())
        } else {
            Err(anyhow!("{:?} is not a directory", directory))
        }
    }
}
