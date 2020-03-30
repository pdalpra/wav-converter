mod encoding;
mod errors;
mod files;
mod opts;
mod tagging;

use crate::errors::{Result, WavToFlacError};
use executors::{crossbeam_workstealing_pool, Executor};
use log::info;
use opts::Opts;
use std::sync::mpsc;
use std::time::Instant;
use structopt::StructOpt;

fn main() -> Result<()> {
    let opts: Opts = Opts::from_args().validate()?;
    setup_logger(&opts);

    let jobs = files::find_files_to_encode(&opts.src, &opts.dest);
    let nb_jobs = jobs.len();
    let compression = opts.compression;

    let progress_bar = setup_progress_bar(&opts, nb_jobs as u64);
    let pool = crossbeam_workstealing_pool::small_pool(num_cpus::get());

    let (tx, rx) = mpsc::channel();
    let before_conversion_start = Instant::now();

    for file in jobs {
        let tx = tx.clone();
        pool.execute(move || {
            tx.send(file.convert_to_flac(compression))
                .expect("Channel should be available");
        });
    }

    for result in rx.iter().take(nb_jobs) {
        if let Some(err) = result.err() {
            info!("Error during conversion: {}", err)
        }
        progress_bar.iter().for_each(|bar| bar.inc(1));
    }

    progress_bar.iter().for_each(|bar| bar.finish());
    pool.shutdown().map_err(WavToFlacError::ExecutorsError)?;

    if nb_jobs != 0 {
        info!(
            "Conversion completed in {}s.",
            before_conversion_start.elapsed().as_secs()
        );
    } else {
        info!("All files are already converted to FLAC.")
    }

    Ok(())
}

fn setup_logger(opts: &Opts) {
    pretty_env_logger::formatted_builder()
        .filter(Some(module_path!()), opts.log_level())
        .format_module_path(false)
        .format_indent(None)
        .init()
}

fn setup_progress_bar(opts: &Opts, nb_jobs: u64) -> Option<indicatif::ProgressBar> {
    if opts.quiet || nb_jobs == 0 {
        None
    } else {
        Some(indicatif::ProgressBar::new(nb_jobs))
    }
}
