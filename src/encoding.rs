use crate::flags::EncodingOptions;
use crate::format::Format;
use crate::tagging;

use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::files::FileMapping;
use anyhow::*;

pub fn convert(mapping: &FileMapping, encoding_options: &EncodingOptions, cover_name: &str, debug: bool) -> Result<()> {
    encode(mapping, &encoding_options, debug)?;

    let cover_file: Option<PathBuf> = mapping
        .target_file
        .parent()
        .map(|parent| parent.join(cover_name))
        .filter(|path| path.exists());

    tagging::tag_file(&mapping.target_file, cover_file)?;

    Ok(())
}

fn encode(mapping: &FileMapping, encoding_options: &EncodingOptions, debug: bool) -> Result<()> {
    let mut cmd = Command::new("ffmpeg");

    if !debug {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    }

    cmd.arg("-i")
        .arg(&mapping.source_file)
        .args(&["-map_metadata", "-1"])
        .args(encoding_arguments(encoding_options))
        .arg(&mapping.target_file);

    let status_code = &cmd.status().map_err(|err| anyhow!("Error while running ffmpeg: {:?}", err))?;

    if status_code.success() {
        Ok(())
    } else {
        Err(anyhow!(
            "Error while running ffmpeg: exited with status code {}",
            status_code.code().unwrap()
        ))
    }
}

fn encoding_arguments(encoding_options: &EncodingOptions) -> Vec<String> {
    let mut arguments = vec!["-c:a".to_string(), encoding_options.format.codec_name().to_string()];

    if let Some(sample_rate) = encoding_options.sample_rate {
        arguments.append(&mut vec!["-ar".to_string(), sample_rate.to_string()]);
    }

    let mut format_specific = match encoding_options.format {
        Format::Flac => vec!["-compression_level".to_string(), encoding_options.compression.to_string()],
        Format::Alac => vec![],
    };
    arguments.append(&mut format_specific);
    arguments
}
