use std::io::{Read, Write};

use super::traits::{DataParser, DataWriter};
use crate::domain::tx::*;
use crate::errors::AppError;

#[derive(Default)]
pub(crate) struct DummyCodec {}
impl DataParser for DummyCodec {
    fn parse<R: Read>(&self, _: R) -> Result<Vec<TxRecord>, AppError> {
        Ok(vec![])
    }
}
impl DataWriter for DummyCodec {
    fn write<W: Write>(&self, _: &mut W, _: &[TxRecord]) -> Result<(), AppError> {
        Ok(())
    }
}
