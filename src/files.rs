use std::collections::HashSet;
use std::fmt::Debug;
use std::fs;
use std::io;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::{error::Error, fs::File};

use anyhow::Result;
use walkdir::{DirEntry, WalkDir};

pub fn find_audio_files_and_covers(
    src_root: &PathBuf,
    dest_root: &PathBuf,
    target_extension: &str,
    cover_name: &str,
) -> (Vec<FileMapping>, Vec<FileMapping>) {
    let (dir_entries, errors): (Vec<DirEntry>, Vec<walkdir::Error>) = partition_result(WalkDir::new(src_root));
    log_errors_if_any(errors);

    let mut wav_files: Vec<PathBuf> = vec![];
    let mut cover_files: Vec<PathBuf> = vec![];

    for entry in dir_entries {
        let path = entry.path();
        if is_valid_wav_file(path) {
            wav_files.push(path.to_path_buf());
        } else if entry.file_name() == cover_name {
            cover_files.push(path.to_path_buf());
        }
    }

    log::info!("Found {} WAV files in {}", wav_files.len(), src_root.to_string_lossy());

    let audio_file_mappings: Vec<FileMapping> = wav_files
        .into_iter()
        .flat_map(|path| {
            on_missing_file(&path, src_root, dest_root, |target_path| {
                target_path.with_extension(target_extension)
            })
        })
        .collect();

    log::info!("Found {} missing files to encode", audio_file_mappings.len());

    let cover_file_mappings: Vec<FileMapping> = cover_files
        .into_iter()
        .flat_map(|path| on_missing_file(&path, src_root, dest_root, std::convert::identity))
        .collect();

    log::info!("Found {} missing covers to copy", cover_file_mappings.len());

    (audio_file_mappings, cover_file_mappings)
}

pub fn create_directories(mappings: &[FileMapping]) -> Result<()> {
    let parents: HashSet<&Path> = mappings.iter().flat_map(|file| file.target_file.parent()).collect();

    let (_, errors): (Vec<_>, Vec<io::Error>) = partition_result(parents.iter().map(fs::create_dir_all));
    log_errors_if_any(errors);

    Ok(())
}

pub fn copy_covers(covers: &[FileMapping]) -> Result<()> {
    let (_, errors): (Vec<_>, Vec<io::Error>) =
        partition_result(covers.iter().map(|mapping| fs::copy(&mapping.source_file, &mapping.target_file)));

    log_errors_if_any(errors);

    Ok(())
}

fn on_missing_file<F>(source_file: &Path, src_root: &PathBuf, dest_root: &Path, map_file_name: F) -> Option<FileMapping>
where
    F: FnOnce(PathBuf) -> PathBuf,
{
    pathdiff::diff_paths(source_file, src_root)
        .map(|relative_path| map_file_name(dest_root.join(relative_path)))
        .filter(|target_path| !target_path.exists())
        .map(|target_path| FileMapping::new(source_file.to_path_buf(), target_path))
}

fn is_valid_wav_file(path: &Path) -> bool {
    File::open(path)
        .map(|mut file| hound::read_wave_header(&mut file).is_ok())
        .unwrap_or(false)
}

fn log_errors_if_any(errors: Vec<impl Error>) {
    if log::log_enabled!(log::Level::Debug) {
        for error in &errors {
            log::debug!("Ignoring file, cause: {}", error)
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

pub struct FileMapping {
    pub source_file: PathBuf,
    pub target_file: PathBuf,
}

impl FileMapping {
    pub fn new(source_file: PathBuf, target_file: PathBuf) -> Self {
        FileMapping { source_file, target_file }
    }
}
