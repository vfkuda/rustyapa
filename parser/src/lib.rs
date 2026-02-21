//! Parser library for financial transaction formats.
//! Supports conversion and comparison through shared domain types.
#![warn(missing_docs)]

/// Codecs for reading and writing supported file formats.
pub mod codecs;
/// Domain model for transaction records.
pub mod domain;
/// Common application-level errors.
pub mod errors;
