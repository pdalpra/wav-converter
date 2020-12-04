mod encoding;
mod files;
mod flags;
mod format;
mod tagging;

use crate::flags::Flags;

use std::sync::{mpsc, Arc};
use std::time::Instant;

use anyhow::*;
use executors::{crossbeam_workstealing_pool, Executor};
use indicatif::{FormattedDuration, ProgressBar, ProgressStyle};
use structopt::StructOpt;

fn main() -> Result<()> {
    let flags: Flags = Flags::from_args().validate()?;
    let debug = flags.debug;
    let encoding_options = Arc::new(flags.encoding_options());
    let target_extension = &encoding_options.format.extension();
    let cover_name = Arc::new(flags.cover.clone());

    setup_logger(&flags);

    let (audio_files, cover_files) = files::find_audio_files_and_covers(&flags.src, &flags.dest, target_extension, &cover_name);

    let nb_files = audio_files.len();

    let progress_bar = setup_progress_bar(&flags, nb_files as u64);
    let thread_pool_size = num_cpus::get();
    let pool = crossbeam_workstealing_pool::dyn_pool(thread_pool_size);

    let (tx, rx) = mpsc::channel();
    let before_conversion_start = Instant::now();

    files::create_directories(&audio_files)?;
    files::copy_covers(&cover_files)?;

    for file in audio_files {
        let encoding_options = Arc::clone(&encoding_options);
        let cover_name = Arc::clone(&cover_name);
        let tx = tx.clone();
        pool.execute(move || {
            tx.send(encoding::convert(&file, &encoding_options, &cover_name, debug))
                .expect("Channel should be available");
        });
    }

    for conversion_result in rx.into_iter().take(nb_files) {
        if let Some(err) = conversion_result.err() {
            log::info!("Error during conversion: {}", err)
        }
        progress_bar.inc(1);
    }

    progress_bar.finish();

    if nb_files != 0 {
        log::info!("Conversion completed in {}s.", FormattedDuration(before_conversion_start.elapsed()));
    } else {
        log::info!(
            "All files are already converted to {}.",
            encoding_options.format.codec_name().to_uppercase()
        )
    }

    pool.shutdown().map_err(|err| anyhow!("Failed to shutdown executors pool: {}", err))
}

fn setup_logger(opts: &Flags) {
    pretty_env_logger::formatted_builder()
        .filter(Some(module_path!()), opts.log_level())
        .format_module_path(false)
        .format_indent(None)
        .init()
}

fn setup_progress_bar(opts: &Flags, nb_files: u64) -> ProgressBar {
    if opts.quiet || nb_files == 0 {
        ProgressBar::hidden()
    } else {
        let progress_style = ProgressStyle::default_bar()
            .template("{pos}/{len} [{bar:60.cyan/blue}] {percent}% (eta: {eta})")
            .progress_chars("#>-");
        ProgressBar::new(nb_files).with_style(progress_style)
    }
}
