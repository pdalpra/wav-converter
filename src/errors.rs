use std::{fmt, io, result};

use metaflac::Error;
use std::fmt::{Debug, Display, Formatter};
use std::iter::FromIterator;
use std::path::PathBuf;

pub type Result<T> = result::Result<T, WavToFlacError>;

#[derive(Debug)]
pub enum WavToFlacError {
    NotADirectory(PathBuf),
    IoError(io::Error),
    WalkError(walkdir::Error),
    WavError(hound::Error),
    ExecutorsError(String),
    EncoderInitError(flac_bound::FlacEncoderInitError),
    EncoderError(flac_bound::FlacEncoderState),
    TaggingError(String),
    TaggingWriteError(metaflac::Error),
}

impl Display for WavToFlacError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            WavToFlacError::NotADirectory(path) => write!(f, "{} is not a directory", path.to_string_lossy()),
            WavToFlacError::IoError(error) => write!(f, "I/O error: {}", error),
            WavToFlacError::WalkError(error) => write!(f, "Error while listing files: {}", error),
            WavToFlacError::WavError(error) => write!(f, "Error while reading WAV file: {}", error),
            WavToFlacError::ExecutorsError(error) => write!(f, "Error in job pool operation: {}", error),
            WavToFlacError::EncoderInitError(error) => write!(f, "Error while initializing encoder: {:?}", error),
            WavToFlacError::EncoderError(error) => write!(f, "Error while encoding file: {:?}", error),
            WavToFlacError::TaggingError(message) => write!(f, "Error while extracting data for tagging: {}", message),
            WavToFlacError::TaggingWriteError(error) => write!(f, "Error while  writing tags: {}", error),
        }
    }
}

impl From<flac_bound::FlacEncoderInitError> for WavToFlacError {
    fn from(error: flac_bound::FlacEncoderInitError) -> Self {
        WavToFlacError::EncoderInitError(error)
    }
}

impl From<flac_bound::FlacEncoderState> for WavToFlacError {
    fn from(error: flac_bound::FlacEncoderState) -> Self {
        WavToFlacError::EncoderError(error)
    }
}

impl From<hound::Error> for WavToFlacError {
    fn from(error: hound::Error) -> Self {
        WavToFlacError::WavError(error)
    }
}

impl From<io::Error> for WavToFlacError {
    fn from(error: io::Error) -> Self {
        WavToFlacError::IoError(error)
    }
}

impl From<metaflac::Error> for WavToFlacError {
    fn from(error: Error) -> Self {
        WavToFlacError::TaggingWriteError(error)
    }
}

impl From<walkdir::Error> for WavToFlacError {
    fn from(error: walkdir::Error) -> Self {
        WavToFlacError::WalkError(error)
    }
}

pub fn partition_result<I, T, E, Successes, Errors>(iterable: I) -> (Successes, Errors)
where
    I: IntoIterator<Item = result::Result<T, E>>,
    T: Debug,
    E: Debug,
    Successes: FromIterator<T>,
    Errors: FromIterator<E>,
{
    let (successes, failures): (Vec<_>, Vec<_>) = iterable.into_iter().partition(|e| e.is_ok());
    let successes = successes.into_iter().map(result::Result::unwrap).collect();
    let failures = failures.into_iter().map(result::Result::unwrap_err).collect();
    (successes, failures)
}
