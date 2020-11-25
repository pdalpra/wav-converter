use crate::tagging;

use std::fs;
use std::path::PathBuf;

use ac_ffmpeg::codec::audio::{AudioTranscoder, ChannelLayout};
use ac_ffmpeg::codec::{AudioCodecParameters, CodecParameters};
use ac_ffmpeg::format::demuxer::{Demuxer, DemuxerWithCodecParameters};
use ac_ffmpeg::format::io::IO;
use ac_ffmpeg::format::muxer::{Muxer, OutputFormat};
use anyhow::*;
use std::fs::{File, OpenOptions};

pub struct JobRequest {
    source_file: PathBuf,
    target_file: PathBuf,
}

impl JobRequest {
    pub fn new(source_file: PathBuf, target_file: PathBuf) -> Self {
        JobRequest {
            source_file,
            target_file,
        }
    }

    pub fn convert(&self, codec: &str) -> Result<()> {
        if let Some(parent) = self.target_file.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut demuxer = self.demux_from_file()?;
        let input_params = demuxer
            .codec_parameters()
            .first()
            .and_then(|raw_params| raw_params.clone().into_audio_codec_parameters())
            .ok_or(anyhow!("Expected a single audio channel, but could not find it"))?;

        let input_params = JobRequest::conversion_parameters(&input_params, "flac")?;
        let output_params = JobRequest::conversion_parameters(&input_params, codec)?;
        let mut transcoder = AudioTranscoder::builder(input_params, output_params.clone())?.build()?;

        let mut muxer = self.mux_to_file(CodecParameters::from(output_params.clone()), codec)?;

        // Push all source packets to the transcoder
        while let Some(packet) = demuxer.take()? {
            transcoder.push(packet)?;
        }

        // Push all transcoder packets to the output file
        while let Some(packet) = transcoder.take()? {
            muxer.push(packet)?;
        }

        tagging::tag_file(&self.target_file)?;

        Ok(())
    }

    fn demux_from_file(&self) -> Result<DemuxerWithCodecParameters<File>> {
        let file = File::open(&self.source_file)
            .map_err(|err| anyhow!("Could not open file {:?}: {}", &self.source_file, err))?;

        Demuxer::builder()
            .build(IO::from_seekable_read_stream(file))?
            .find_stream_info(None)
            .map_err(|(_, err)| anyhow!("Could not demux WAV file {:?}: {}", &self.source_file, err))
    }

    fn mux_to_file(&self, parameters: CodecParameters, codec: &str) -> Result<Muxer<File>> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.target_file)
            .map_err(|err| anyhow!("Could not open file {:?}: {}", &self.target_file, err))?;

        let output_format = OutputFormat::find_by_name(codec).unwrap();

        let mut muxer = Muxer::builder();
        muxer.add_stream(&parameters)?;

        muxer
            .build(IO::from_seekable_write_stream(file), output_format)
            .map_err(|err| anyhow!("Could not open muxer for {:?}: {}", &self.target_file, err))
    }

    fn conversion_parameters(
        input_codec_parameters: &AudioCodecParameters,
        codec: &str,
    ) -> Result<AudioCodecParameters> {
        let audio_parameters = AudioCodecParameters::builder(codec)?
            .sample_rate(input_codec_parameters.sample_rate())
            .sample_format(input_codec_parameters.sample_format())
            .channel_layout(ChannelLayout::from_channels(2).unwrap())
            .build();

        Ok(audio_parameters)
    }
}
