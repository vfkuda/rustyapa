use super::base::TxFieldKey;
use super::errors::{ParserContext, ParserCtxBehavior, ParserError};
use super::traits::{DataParser, DataWriter};
use super::utils::unquote;
use crate::codecs::errors::IoCtxBehavior;
use crate::domain::tx::*;
use crate::errors::AppError;
use std::io::{BufRead, BufReader, Read, Write};

const FIELD_KV_DELIMITER: char = ':';
const COMMENT_SYMBOL_1LINE: char = '#';

struct RecordBuilder {
    is_dirty: bool,
    id: Option<TxIdType>,
    kind: Option<TxKind>,
    from: Option<AccountType>,
    to: Option<AccountType>,
    amount: Option<i64>,
    ts: Option<TxTimestamp>,
    status: Option<TxStatus>,
    description: Option<String>,
}
impl RecordBuilder {
    fn new() -> Self {
        Self {
            is_dirty: false,
            id: None,
            kind: None,
            from: None,
            to: None,
            amount: None,
            ts: None,
            status: None,
            description: None,
        }
    }

    fn is_key_already_present(&self, field_key: &TxFieldKey) -> bool {
        match field_key {
            TxFieldKey::Id => self.id.is_some(),
            TxFieldKey::TxKind => self.kind.is_some(),
            TxFieldKey::FromUserId => self.from.is_some(),
            TxFieldKey::ToUserId => self.to.is_some(),
            TxFieldKey::Amount => self.amount.is_some(),
            TxFieldKey::Timestamp => self.ts.is_some(),
            TxFieldKey::Status => self.status.is_some(),
            TxFieldKey::Description => self.description.is_some(),
        }
    }

    fn set_field_value(&mut self, field_key: TxFieldKey, value: &str) -> Result<(), ParserError> {
        // println!("parse from line [{:?}]-->{}<--", field_key, value);
        if self.is_key_already_present(&field_key) {
            return Err(ParserError::Duplicate(field_key));
        }
        self.is_dirty = true;
        match field_key {
            TxFieldKey::Id => self.id = Some(value.parse()?),
            TxFieldKey::TxKind => self.kind = Some(value.parse()?),
            TxFieldKey::FromUserId => self.from = Some(value.parse()?),
            TxFieldKey::ToUserId => self.to = Some(value.parse()?),
            TxFieldKey::Amount => self.amount = Some(value.parse()?),
            TxFieldKey::Timestamp => self.ts = Some(value.parse()?),
            TxFieldKey::Status => self.status = Some(value.parse()?),
            TxFieldKey::Description => self.description = Some(unquote(value)?.to_string()),
        };
        Ok(())
    }
    fn parse_field_from_line(&mut self, line: &str) -> Result<(), ParserError> {
        // split string to key=value pair and save to buffer
        let (key, value) = line
            .split_once(FIELD_KV_DELIMITER)
            .ok_or(ParserError::NoFieldDelimiter)?;
        let field_key = key.trim().parse::<TxFieldKey>()?;
        self.set_field_value(field_key, value.trim())?;
        Ok(())
    }
    fn finalize(&mut self) -> Result<TxRecord, ParserError> {
        let tx = TxRecord {
            id: self
                .id
                .take()
                .ok_or(ParserError::MissingField(TxFieldKey::Id))?,
            kind: self
                .kind
                .take()
                .ok_or(ParserError::MissingField(TxFieldKey::TxKind))?,
            from: self
                .from
                .take()
                .ok_or(ParserError::MissingField(TxFieldKey::FromUserId))?,
            to: self
                .to
                .take()
                .ok_or(ParserError::MissingField(TxFieldKey::ToUserId))?,
            amount: self
                .amount
                .take()
                .ok_or(ParserError::MissingField(TxFieldKey::Amount))?,
            ts: self
                .ts
                .take()
                .ok_or(ParserError::MissingField(TxFieldKey::Timestamp))?,
            status: self
                .status
                .take()
                .ok_or(ParserError::MissingField(TxFieldKey::Status))?,
            description: self
                .description
                .take()
                .ok_or(ParserError::MissingField(TxFieldKey::Description))?,
        };
        Ok(tx)
    }
}

#[derive(Default)]
pub(crate) struct TextCodec;
impl TextCodec {
    fn write_kv_pair(
        &self,
        w: &mut dyn Write,
        field_key: TxFieldKey,
        field_value: &str,
    ) -> Result<(), AppError> {
        writeln!(w, "{}{} {}", field_key, FIELD_KV_DELIMITER, field_value).add_write_ctx()
    }
    fn write_single_record(&self, w: &mut dyn Write, tx: &TxRecord) -> Result<(), AppError> {
        self.write_kv_pair(w, TxFieldKey::Id, &tx.id.to_string())?;
        self.write_kv_pair(w, TxFieldKey::TxKind, &tx.kind.to_string())?;
        self.write_kv_pair(w, TxFieldKey::FromUserId, &tx.from.to_string())?;
        self.write_kv_pair(w, TxFieldKey::ToUserId, &tx.to.to_string())?;
        self.write_kv_pair(w, TxFieldKey::Amount, &tx.amount.to_string())?;
        self.write_kv_pair(w, TxFieldKey::Timestamp, &tx.ts.to_string())?;
        self.write_kv_pair(w, TxFieldKey::Status, &tx.status.to_string())?;
        self.write_kv_pair(
            w,
            TxFieldKey::Description,
            &format!("\"{}\"", &tx.description),
        )?;
        Ok(())
    }
}
impl DataParser for TextCodec {
    fn parse<R: Read>(&self, r: R) -> Result<Vec<TxRecord>, AppError> {
        let mut result = Vec::new();
        let mut record_builder = RecordBuilder::new();
        let mut line_num: usize = 0;
        let mut input_line: String = "".to_string();
        for line_res in BufReader::new(r).lines() {
            line_num += 1;
            input_line = line_res.map_err(|e| AppError::ReadError(e))?;
            // println!("{}: {}", line_num, input_line);
            let line = &input_line.trim();

            // skip comments
            if let Some(first_char) = line.chars().nth(0) {
                if COMMENT_SYMBOL_1LINE == first_char {
                    continue;
                }
            }

            // if line is empty - assemble the record
            if line.is_empty() {
                if record_builder.is_dirty {
                    result.push(record_builder.finalize().add_parser_ctx(
                        ParserContext::with_line_number_and_line(line_num, input_line.clone()),
                    )?);
                }
                record_builder = RecordBuilder::new();
                continue;
            }
            record_builder.parse_field_from_line(line).add_parser_ctx(
                ParserContext::with_line_number_and_line(line_num, input_line.clone()),
            )?
        }

        // still some fields in the builder? -> assemble the record
        if record_builder.is_dirty {
            result.push(record_builder.finalize().add_parser_ctx(
                ParserContext::with_line_number_and_line(line_num, input_line.clone()),
            )?);
        }
        Ok(result)
    }
}

impl DataWriter for TextCodec {
    fn write<W: Write>(&self, w: &mut W, data: &[TxRecord]) -> Result<(), AppError> {
        for tx in data {
            self.write_single_record(w, tx)?;
            writeln!(w).add_write_ctx()?;
        }
        Ok(())
    }
}
