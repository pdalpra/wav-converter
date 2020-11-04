use crate::tagging;

use std::fs;
use std::path::PathBuf;

use anyhow::*;
use flac_bound::{FlacEncoder, FlacEncoderConfig};
use hound::{WavReader, WavSpec};

#[derive(Debug)]
pub struct Job {
    source_file: PathBuf,
    target_file: PathBuf,
}

impl Job {
    pub fn new(source_file: PathBuf, target_file: PathBuf) -> Self {
        Job {
            source_file,
            target_file,
        }
    }

    pub fn convert_to_flac(&self, compression: u8) -> Result<()> {
        if let Some(parent) = self.target_file.parent() {
            fs::create_dir_all(parent)?;
        }
        let (spec, samples) = self.read_wav_file()?;
        self.flac_encode(&spec, &samples, compression)?;
        tagging::tag_file(&self.target_file)?;

        Ok(())
    }

    fn read_wav_file(&self) -> Result<(WavSpec, Vec<i32>)> {
        let mut reader = WavReader::open(&self.source_file)?;
        let spec = reader.spec();
        let samples = reader.samples().map(|s| s.unwrap()).collect::<Vec<i32>>();
        Ok((spec, samples))
    }

    fn flac_encode(&self, spec: &WavSpec, samples: &[i32], compression: u8) -> Result<()> {
        let nb_channels = spec.channels;
        let encoder = FlacEncoder::new().ok_or(anyhow!("Failed to create a FLAC encoder"))?;
        let mut encoder = Self::configure_encoder(encoder, &spec, compression)
            .init_file(&self.target_file)
            .map_err(|err| anyhow!("Error while initializing encoder: {:?}", err))?;

        let mut channels: Vec<Vec<i32>> = vec![Vec::new(); nb_channels as usize];
        for (index, sample) in samples.iter().enumerate() {
            channels[index % (nb_channels as usize)].push(*sample);
        }
        let mut channel_slices = Vec::with_capacity(nb_channels as usize);
        channels.iter().for_each(|channel| channel_slices.push(&channel[..]));

        (&mut encoder)
            .process(&channel_slices)
            .map_err(|_| anyhow!("Error during FLAC encoding: {:?}", encoder.state()))?;

        encoder
            .finish()
            .map_err(|encoder| anyhow!("Failed to finish FLAC encoding: {:?}", encoder.state()))?;
        Ok(())
    }

    fn configure_encoder(encoder: FlacEncoderConfig, spec: &WavSpec, compression: u8) -> FlacEncoderConfig {
        encoder
            .compression_level(compression as u32)
            .sample_rate(spec.sample_rate)
            .bits_per_sample(spec.bits_per_sample as u32)
            .channels(spec.channels as u32)
            .verify(true)
    }
}
