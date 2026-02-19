/// Shared format enums and field mapping utilities.
pub mod base;
/// Binary format codec implementation.
pub mod binary;
/// CSV format codec implementation.
pub mod csv;
/// Stub codec used for testing and wiring.
pub mod dummy;
/// Parsing and IO helper error types.
pub mod errors;
/// Text format codec implementation.
pub mod text;
/// Generic parse/write traits for codecs.
pub mod traits;
/// Internal helper functions used by codecs.
mod utils;
