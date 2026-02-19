use super::base::TxFieldKey;
use crate::errors::AppError;
use std::{fmt::Display, num::ParseIntError};

/// Parser-level errors before they are wrapped into [`AppError`].
#[derive(Debug)]
pub enum ParserError {
    /// Required field is missing in the record.
    MissingField(TxFieldKey),
    /// Unknown field key was met in input.
    UnparsableKey(String),
    /// Field value cannot be parsed into expected type.
    UnparsableValue(String),
    /// Same field was provided more than one time.
    Duplicate(TxFieldKey),
    /// Key-value delimiter is absent in text line.
    NoFieldDelimiter,
    /// String value expected to be wrapped in double quotes.
    ShellBeQuoted(String),
    /// File header is invalid for current format.
    InvalidFileHeader,
    /// Record header signature is invalid.
    InvalidRecordHeader(String),
    /// Record does not have all required fields.
    IncompleteRecord,
}

impl std::error::Error for ParserError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            _ => None,
        }
    }
}

impl From<ParseIntError> for ParserError {
    fn from(value: ParseIntError) -> Self {
        Self::UnparsableValue(value.to_string())
    }
}

/// Additional context attached to parser errors.
#[derive(Debug)]
pub enum ParserContext {
    LineNumAndLine {
        line_num: usize,
        line: String,
    },
    Position {
        position: usize,
    },
    PositionAndField {
        position: usize,
        field_key: TxFieldKey,
    },
}
impl ParserContext {
    pub(super) fn with_line_number_and_line(line_num: usize, line: String) -> Self {
        Self::LineNumAndLine { line_num, line }
    }
    pub(super) fn with_position(position: usize) -> Self {
        Self::Position { position }
    }
    pub(super) fn with_position_and_field_key(position: usize, field_key: TxFieldKey) -> Self {
        Self::PositionAndField {
            position,
            field_key,
        }
    }
}

impl Display for ParserContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserContext::LineNumAndLine { line_num, line } => {
                writeln!(f, "line #{}, content: `{}`", line_num, line)
            }
            ParserContext::Position { position } => {
                writeln!(f, "position #{}", position)
            }
            ParserContext::PositionAndField {
                position,
                field_key,
            } => {
                writeln!(
                    f,
                    "position #{}, field being parsed: `{}`",
                    position, field_key
                )
            }
        }
    }
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::MissingField(field_key) => {
                write!(f, "required field {} is missing", field_key)
            }
            ParserError::UnparsableKey(key_name) => {
                write!(f, "unknown key {}", key_name)
            }
            ParserError::UnparsableValue(value) => {
                write!(f, "value {} can't be parsed", value)
            }
            ParserError::ShellBeQuoted(str) => {
                write!(f, "string ->{}<- shall be double quoted", str)
            }
            ParserError::Duplicate(field_key) => {
                write!(f, "field {} has duplicate", field_key)
            }
            ParserError::NoFieldDelimiter => {
                write!(f, "key-value delimiter is expected")
            }
            ParserError::InvalidFileHeader => {
                write!(f, "invalid file header")
            }
            ParserError::IncompleteRecord => {
                write!(f, "incomplete record (doesn't have all required fields)")
            }
            ParserError::InvalidRecordHeader(instead) => {
                write!(f, "invalid record header {:?}", instead)
            }
        }
    }
}

pub(super) trait IoCtxBehavior<T> {
    fn add_read_ctx(self) -> Result<T, AppError>;
    fn add_write_ctx(self) -> Result<T, AppError>;
}

impl<T> IoCtxBehavior<T> for Result<T, std::io::Error> {
    fn add_read_ctx(self) -> Result<T, AppError> {
        self.map_err(|e| AppError::ReadError(e))
    }
    fn add_write_ctx(self) -> Result<T, AppError> {
        self.map_err(|e| AppError::WriteError(e))
    }
}

pub(super) trait ParserCtxBehavior<T> {
    fn add_parser_ctx(self, ctx: ParserContext) -> Result<T, AppError>;
}

impl<T> ParserCtxBehavior<T> for Result<T, ParserError> {
    fn add_parser_ctx(self, ctx: ParserContext) -> Result<T, AppError> {
        self.map_err(|e| AppError::ParsingError {
            context: ctx,
            source: e,
        })
    }
}
