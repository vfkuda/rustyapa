use crate::domain::tx::*;
use crate::errors::AppError;
use std::io::{Read, Write};

/// Parses transaction records from any input implementing [`Read`].
pub trait DataParser {
    /// Reads all records from stream and returns parsed domain objects.
    fn parse<R: Read>(&self, r: R) -> Result<Vec<TxRecord>, AppError>;
}
/// Writes transaction records to any output implementing [`Write`].
pub trait DataWriter {
    /// Serializes all provided records into writer in codec-specific format.
    fn write<W: Write>(&self, w: &mut W, data: &[TxRecord]) -> Result<(), AppError>;
}
