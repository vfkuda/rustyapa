use std::io::{BufRead, BufReader, Read, Write};

use super::traits::{DataParser, DataWriter};
use super::utils::unquote;

use crate::codecs::errors::{IoCtxBehavior, ParserContext, ParserError};
use crate::domain::tx::*;
use crate::errors::AppError;

const HEADER_SIGNATURE: &str =
    "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION";
const CSV_DELIMITER: char = ',';

const FIELDS_COUNT: usize = 8;

const TX_ID: usize = 0;
const TX_TYPE: usize = 1;
const FROM_USER_ID: usize = 2;
const TO_USER_ID: usize = 3;
const AMOUNT: usize = 4;
const TIMESTAMP: usize = 5;
const STATUS: usize = 6;
const DESCRIPTION: usize = 7;

#[derive(Default)]
pub(crate) struct CsvCodec;
impl CsvCodec {
    fn parse_csv_line(&self, line: &str) -> Result<TxRecord, ParserError> {
        let values: Vec<&str> = line.split(CSV_DELIMITER).map(str::trim).collect();
        if values.len() != FIELDS_COUNT {
            return Err(ParserError::IncompleteRecord);
        }

        Ok(TxRecord {
            id: values[TX_ID].parse()?,
            kind: values[TX_TYPE].parse()?,
            from: values[FROM_USER_ID].parse()?,
            to: values[TO_USER_ID].parse()?,
            amount: values[AMOUNT].parse()?,
            ts: values[TIMESTAMP].parse()?,
            status: values[STATUS].parse()?,
            description: unquote(values[DESCRIPTION])?.to_string(),
        })
    }

    fn write_single_record(&self, w: &mut dyn Write, tx: &TxRecord) -> Result<(), AppError> {
        let mut values = Vec::with_capacity(FIELDS_COUNT);
        values.push(tx.id.to_string());
        values.push(tx.kind.to_string());
        values.push(tx.from.to_string());
        values.push(tx.to.to_string());
        values.push(tx.amount.to_string());
        values.push(tx.ts.to_string());
        values.push(tx.status.to_string());
        values.push(format!("\"{}\"", tx.description));

        // self-check
        assert!(values.len() == FIELDS_COUNT);

        writeln!(w, "{}", values.join(&CSV_DELIMITER.to_string())).add_write_ctx()
    }
}
impl DataParser for CsvCodec {
    fn parse<R: Read>(&self, r: R) -> Result<Vec<TxRecord>, AppError> {
        let mut result = Vec::new();

        let mut lines = BufReader::new(r).lines().enumerate();
        // check header
        if let Some((line_num, header_res)) = lines.next() {
            let header = header_res.map_err(|e| AppError::ReadError(e))?;
            if HEADER_SIGNATURE != header {
                return Err(AppError::ParsingError {
                    context: ParserContext::with_line_number_and_line(line_num, header),
                    source: ParserError::InvalidFileHeader,
                });
            }
        }

        // read/parse records line by line
        for (line_num, line_res) in lines {
            let input_line = line_res.map_err(|e| AppError::ReadError(e))?;
            let line = &input_line.trim();
            result.push(
                self.parse_csv_line(line)
                    .map_err(|e| AppError::ParsingError {
                        context: ParserContext::with_line_number_and_line(
                            line_num,
                            input_line.clone(),
                        ),
                        source: e,
                    })?,
            );
        }
        Ok(result)
    }
}

impl DataWriter for CsvCodec {
    fn write<W: Write>(&self, w: &mut W, data: &[TxRecord]) -> Result<(), AppError> {
        writeln!(w, "{}", HEADER_SIGNATURE).add_write_ctx()?;
        for tx in data {
            self.write_single_record(w, tx)?;
        }
        Ok(())
    }
}
