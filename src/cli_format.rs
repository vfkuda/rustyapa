use std::fmt::Display;

use clap::ValueEnum;
use parser::codecs::base::Codec;

/// Supported formats
#[derive(Clone, Debug, ValueEnum)]
pub enum Format {
    /// Binary file format.
    Binary,
    /// Text file format.
    Text,
    /// CSV file format.
    Csv,
}
impl Format {
    /// Returns format-specific codec.
    pub fn codec(&self) -> Codec {
        match &self {
            Format::Binary => Codec::BinaryCodec,
            Format::Text => Codec::TextCodec,
            Format::Csv => Codec::CsvCodec,
        }
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::Binary => write!(f, "binary"),
            Format::Text => write!(f, "text"),
            Format::Csv => write!(f, "csv"),
        }
    }
}
