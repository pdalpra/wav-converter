use std::path::PathBuf;

use structopt::StructOpt;

use crate::errors::{Result, WavToFlacError};
use log::LevelFilter;

#[derive(Debug, StructOpt)]
pub struct Opts {
    /// Silence all output
    #[structopt(short, long)]
    pub quiet: bool,

    /// Enable debug logs
    #[structopt(short, long)]
    pub debug: bool,

    #[structopt(short, long, default_value = "8")]
    pub compression: u8,

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
        Self::validate_directory(&self.dest)?;
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
            Err(WavToFlacError::NotADirectory(directory.clone()))
        }
    }
}
