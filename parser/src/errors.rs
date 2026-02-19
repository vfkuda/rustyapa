use std::{fmt::Display, io};

use crate::codecs::errors::ParserContext;
use crate::codecs::errors::ParserError;

/// application error for IO and parsing operations.
#[derive(Debug)]
pub enum AppError {
    /// Error happened while reading input stream.
    ReadError(io::Error),
    /// Error happened while writing output stream.
    WriteError(io::Error),
    /// Error happened while parsing with additional context.
    ParsingError {
        /// Position context where parsing failed.
        context: ParserContext,
        /// Concrete parsing error cause.
        source: ParserError,
    },
}

impl std::error::Error for AppError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            AppError::ReadError(e) => Some(e),
            AppError::WriteError(e) => Some(e),
            AppError::ParsingError { context: _, source } => Some(source),
        }
    }
}

impl Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::ReadError(e) => write!(f, "read error, {}", e),
            AppError::WriteError(e) => write!(f, "write error, {}", e),
            AppError::ParsingError { context, source } => {
                writeln!(f, "{}:\n{}", source.to_string(), context.to_string(),)
            }
        }
    }
}
