use std::fmt::Display;
use std::io::{Read, Write};
use std::str::FromStr;

use crate::domain::tx::*;
use crate::errors::AppError;

use super::binary::BinaryCodec;
use super::csv::CsvCodec;
use super::dummy::DummyCodec;
use super::errors::ParserError;
use super::text::TextCodec;
use super::traits::*;

/// Supported Codecs factory.
#[derive(Clone, Debug)]
pub enum Codec {
    /// Codec for Binary format.
    BinaryCodec,
    /// Codec for Text format.
    TextCodec,
    /// Codec for CSV format.
    CsvCodec,
    /// Dummy format used for no-op behavior.
    DummyCodec,
}

impl Codec {
    /// Parses records from input stream using selected codec.
    pub fn parse<R: Read>(&self, r: R) -> Result<Vec<TxRecord>, AppError> {
        match self {
            Codec::BinaryCodec => BinaryCodec::default().parse(r),
            Codec::TextCodec => TextCodec::default().parse(r),
            Codec::CsvCodec => CsvCodec::default().parse(r),
            Codec::DummyCodec => DummyCodec::default().parse(r),
        }
    }
    /// Writes records to output stream using selected codec.
    pub fn write<W: Write>(&self, w: &mut W, data: &[TxRecord]) -> Result<(), AppError> {
        match self {
            Codec::BinaryCodec => BinaryCodec::default().write(w, data),
            Codec::TextCodec => TextCodec::default().write(w, data),
            Codec::CsvCodec => CsvCodec::default().write(w, data),
            Codec::DummyCodec => DummyCodec::default().write(w, data),
        }
    }
}

//
// parsing implementations for tx types
//
impl FromStr for TxIdType {
    type Err = ParserError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<u64>()?;
        Ok(TxIdType(value))
    }
}

impl FromStr for AccountType {
    type Err = ParserError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = s.parse::<u64>()?;
        Ok(AccountType(value))
    }
}

impl FromStr for TxKind {
    type Err = ParserError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DEPOSIT" => Ok(TxKind::Deposit),
            "TRANSFER" => Ok(TxKind::Transfer),
            "WITHDRAWAL" => Ok(TxKind::Withdrawal),
            _ => Err(ParserError::UnparsableValue(s.into())),
        }
    }
}
impl FromStr for TxTimestamp {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TxTimestamp::parse_timestamp(s)
    }
}

impl FromStr for TxStatus {
    type Err = ParserError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SUCCESS" => Ok(TxStatus::Success),
            "FAILURE" => Ok(TxStatus::Failure),
            "PENDING" => Ok(TxStatus::Pending),
            _ => Err(ParserError::UnparsableValue(s.to_string())),
        }
    }
}

//
// Transaction fields composite types and display/parse for them
//
#[derive(Debug)]
/// Field names shared between text and csv formats.
pub enum TxFieldKey {
    /// `TX_ID` field.
    Id,
    /// `TX_TYPE` field.
    TxKind,
    /// `FROM_USER_ID` field.
    FromUserId,
    /// `TO_USER_ID` field.
    ToUserId,
    /// `AMOUNT` field.
    Amount,
    /// `TIMESTAMP` field.
    Timestamp,
    /// `STATUS` field.
    Status,
    /// `DESCRIPTION` field.
    Description,
}
impl Display for TxFieldKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxFieldKey::Id => write!(f, "TX_ID"),
            TxFieldKey::TxKind => write!(f, "TX_TYPE"),
            TxFieldKey::FromUserId => write!(f, "FROM_USER_ID"),
            TxFieldKey::ToUserId => write!(f, "TO_USER_ID"),
            TxFieldKey::Amount => write!(f, "AMOUNT"),
            TxFieldKey::Timestamp => write!(f, "TIMESTAMP"),
            TxFieldKey::Status => write!(f, "STATUS"),
            TxFieldKey::Description => write!(f, "DESCRIPTION"),
        }
    }
}
impl FromStr for TxFieldKey {
    type Err = ParserError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "TX_ID" => Ok(TxFieldKey::Id),
            "TX_TYPE" => Ok(TxFieldKey::TxKind),
            "FROM_USER_ID" => Ok(TxFieldKey::FromUserId),
            "TO_USER_ID" => Ok(TxFieldKey::ToUserId),
            "AMOUNT" => Ok(TxFieldKey::Amount),
            "TIMESTAMP" => Ok(TxFieldKey::Timestamp),
            "STATUS" => Ok(TxFieldKey::Status),
            "DESCRIPTION" => Ok(TxFieldKey::Description),
            _ => Err(ParserError::UnparsableKey(s.into())),
        }
    }
}
