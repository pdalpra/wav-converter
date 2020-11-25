mod encoding;
mod files;
mod opts;
mod tagging;

use crate::opts::Opts;

use std::sync::mpsc;
use std::time::Instant;

use anyhow::*;
use executors::{crossbeam_workstealing_pool, Executor};
use indicatif::ProgressBar;
use log::info;
use structopt::StructOpt;

fn main() -> Result<()> {
    let opts: Opts = Opts::from_args().validate()?;
    setup_logger(&opts);

    let jobs = files::find_files_to_encode(&opts.src, &opts.dest);
    let nb_jobs = jobs.len();
    let compression = opts.compression;

    let progress_bar = setup_progress_bar(&opts, nb_jobs as u64);
    let thread_pool_size = num_cpus::get();
    let pool = crossbeam_workstealing_pool::dyn_pool(thread_pool_size);

    let (tx, rx) = mpsc::channel();
    let before_conversion_start = Instant::now();

    for file in jobs {
        let tx = tx.clone();
        pool.execute(move || {
            tx.send(file.convert("alac")).expect("Channel should be available");
        });
    }

    for result in rx.iter().take(nb_jobs) {
        if let Some(err) = result.err() {
            info!("Error during conversion: {}", err)
        }
        progress_bar.inc(1);
    }

    progress_bar.finish();
    pool.shutdown()
        .map_err(|err| anyhow!("Failed to shutdown executors pool: {}", err))?;

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
    //ac_ffmpeg::set_log_callback(|_, _| ()); // Disable ffmpeg logging
    pretty_env_logger::formatted_builder()
        .filter(Some(module_path!()), opts.log_level())
        .format_module_path(false)
        .format_indent(None)
        .init()
}

fn setup_progress_bar(opts: &Opts, nb_jobs: u64) -> ProgressBar {
    if opts.quiet || nb_jobs == 0 {
        ProgressBar::hidden()
    } else {
        ProgressBar::new(nb_jobs)
    }
}
