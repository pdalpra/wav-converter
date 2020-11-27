use crate::encoding::Job;

use std::fmt::Debug;
use std::iter::FromIterator;
use std::{error::Error, fs::File, io, path::PathBuf};

use log::{debug, info, log_enabled, Level};
use walkdir::WalkDir;

pub fn find_files_to_encode(src: &PathBuf, dest: &PathBuf) -> Vec<Job> {
    let file_walker = WalkDir::new(src).follow_links(true);
    let (entries, walk_errors): (Vec<_>, Vec<_>) = partition_result(file_walker.into_iter());
    log_errors_if_any(walk_errors);

    let detected_wav_files = entries.into_iter().map(detect_wav_file);
    let (wav_files, wav_detection_errors): (Vec<_>, Vec<_>) = partition_result(detected_wav_files);
    log_errors_if_any(wav_detection_errors);

    let wav_files: Vec<_> = wav_files.into_iter().flatten().collect();
    info!("Found {} WAV files in {}", wav_files.len(), src.to_string_lossy());

    let files_to_encode: Vec<_> = wav_files
        .into_iter()
        .flat_map(|wav_file| build_encoding_job(wav_file, src, dest))
        .collect();

    info!("Found {} missing files to encode", files_to_encode.len());

    files_to_encode
}

fn build_encoding_job(wav_file: PathBuf, src: &PathBuf, dest: &PathBuf) -> Option<Job> {
    pathdiff::diff_paths(wav_file.as_path(), src)
        .map(|relative_path| dest.join(relative_path).with_extension("flac"))
        .filter(|flac_file| !flac_file.exists())
        .map(|flac_file| Job::new(wav_file, flac_file))
}

fn detect_wav_file(dir_entry: walkdir::DirEntry) -> Result<Option<PathBuf>, io::Error> {
    let path = dir_entry.path();
    let file = &mut File::open(path)?;
    Ok(hound::read_wave_header(file).ok().map(|_| path.to_path_buf()))
}

fn log_errors_if_any(errors: Vec<impl Error>) {
    if log_enabled!(Level::Debug) {
        for error in &errors {
            debug!("Ignoring file, cause: {}", error)
        }
    }
}

fn partition_result<I, T, E, Successes, Errors>(iterable: I) -> (Successes, Errors)
where
    I: IntoIterator<Item = Result<T, E>>,
    T: Debug,
    E: Debug,
    Successes: FromIterator<T>,
    Errors: FromIterator<E>,
{
    let (successes, failures): (Vec<_>, Vec<_>) = iterable.into_iter().partition(|e| e.is_ok());
    let successes = successes.into_iter().map(Result::unwrap).collect();
    let failures = failures.into_iter().map(Result::unwrap_err).collect();
    (successes, failures)
}
