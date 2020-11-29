use crate::encoding::FileToConvert;

use std::fmt::Debug;
use std::iter::FromIterator;
use std::{error::Error, fs::File, path::PathBuf};

use log::{debug, info, log_enabled, Level};
use walkdir::{DirEntry, WalkDir};

pub fn find_files_to_convert(src: &PathBuf, dest: &PathBuf, target_extension: &str) -> Vec<FileToConvert> {
    let file_walker = WalkDir::new(src);
    let (dir_entries, walk_errors): (Vec<DirEntry>, Vec<_>) = partition_result(file_walker.into_iter());
    log_errors_if_any(walk_errors);

    let detected_wav_files = dir_entries.into_iter().map(detect_wav_file);
    let (wav_files, wav_detection_errors): (Vec<PathBuf>, Vec<_>) = partition_result(detected_wav_files);
    log_errors_if_any(wav_detection_errors);

    info!("Found {} WAV files in {}", wav_files.len(), src.to_string_lossy());

    let files_to_encode: Vec<FileToConvert> = wav_files
        .into_iter()
        .flat_map(|wav_file| convert_if_missing(wav_file, src, dest, target_extension))
        .collect();

    info!("Found {} missing files to encode", files_to_encode.len());

    files_to_encode
}

fn convert_if_missing(
    wav_file: PathBuf,
    src: &PathBuf,
    dest: &PathBuf,
    target_extension: &str,
) -> Option<FileToConvert> {
    pathdiff::diff_paths(wav_file.as_path(), src)
        .map(|relative_path| dest.join(relative_path).with_extension(target_extension))
        .filter(|converted_file_path| !converted_file_path.exists())
        .map(|converted_file_path| FileToConvert::new(wav_file, converted_file_path))
}

fn detect_wav_file(dir_entry: walkdir::DirEntry) -> Result<PathBuf, hound::Error> {
    let path = dir_entry.path();
    let file = &mut File::open(path)?;
    hound::read_wave_header(file).map(|_| path.to_path_buf())
}

fn log_errors_if_any(errors: Vec<impl Error>) {
    if log_enabled!(Level::Debug) {
        for error in &errors {
            debug!("Ignoring file, cause: {}", error)
        }
    }
}

fn partition_result<T, E, I, Successes, Errors>(iterable: I) -> (Successes, Errors)
where
    T: Debug,
    E: Debug,
    I: IntoIterator<Item = Result<T, E>>,
    Successes: FromIterator<T>,
    Errors: FromIterator<E>,
{
    let (successes, failures): (Vec<I::Item>, Vec<I::Item>) = iterable.into_iter().partition(|e| e.is_ok());
    let successes = successes.into_iter().map(Result::unwrap).collect();
    let failures = failures.into_iter().map(Result::unwrap_err).collect();
    (successes, failures)
}
