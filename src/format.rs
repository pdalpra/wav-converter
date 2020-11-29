use std::str::FromStr;

use anyhow::anyhow;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Format {
    Flac,
    Alac,
}

impl Format {
    const VARIANTS: [Format; 2] = [Format::Flac, Format::Alac];
    pub const DEFAULT_FLAC_COMPRESSION: u8 = 4;

    pub fn codec_name(&self) -> &str {
        match &self {
            Format::Flac => "flac",
            Format::Alac => "alac",
        }
    }

    pub fn extension(&self) -> &str {
        match self {
            Format::Flac => "flac",
            Format::Alac => "m4a",
        }
    }
}

impl FromStr for Format {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "flac" => Ok(Format::Flac),
            "alac" => Ok(Format::Alac),
            format => {
                let supported_formats: Vec<&str> = Format::VARIANTS.iter().map(|f| f.codec_name()).collect();
                Err(anyhow!(
                    "Unsupported format: {} (supported formats: {})",
                    format,
                    supported_formats.join(", ")
                ))
            }
        }
    }
}
