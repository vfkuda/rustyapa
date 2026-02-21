use std::io::{Read, Write};

use super::errors::IoCtxBehavior;
use super::errors::{ParserContext, ParserCtxBehavior, ParserError};
use super::traits::*;
use crate::codecs::base::TxFieldKey;
use crate::domain::tx::*;
use crate::errors::AppError;

const RECORD_MAGIC: [u8; 4] = *b"YPBN";
const MINIMUM_RECORD_SIZE: u32 = 8 + 1 + 8 + 8 + 8 + 8 + 1 + 4;

#[derive(Default)]
pub(crate) struct BinaryCodec {}
impl BinaryCodec {
    fn bytes_to_hex(&self, bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{:02X}", b)).collect()
    }

    fn read_u32_be<R: Read>(&self, r: &mut R) -> Result<u32, AppError> {
        let mut b = [0u8; 4];
        r.read_exact(&mut b).add_read_ctx()?;
        Ok(u32::from_be_bytes(b))
    }
    fn read_u64_be<R: Read>(&self, r: &mut R) -> Result<u64, AppError> {
        let mut b = [0u8; 8];
        r.read_exact(&mut b).add_read_ctx()?;
        Ok(u64::from_be_bytes(b))
    }
    fn read_i64_be<R: Read>(&self, r: &mut R) -> Result<i64, AppError> {
        let mut b = [0u8; 8];
        r.read_exact(&mut b).add_read_ctx()?;
        Ok(i64::from_be_bytes(b))
    }
    fn write_u32_be<W: Write>(&self, w: &mut W, v: u32) -> Result<(), AppError> {
        w.write_all(&v.to_be_bytes()).add_write_ctx()?;
        Ok(())
    }
    fn write_u64_be<W: Write>(&self, w: &mut W, v: u64) -> Result<(), AppError> {
        w.write_all(&v.to_be_bytes()).add_write_ctx()?;
        Ok(())
    }
    fn write_i64_be<W: Write>(&self, w: &mut W, v: i64) -> Result<(), AppError> {
        w.write_all(&v.to_be_bytes()).add_write_ctx()?;
        Ok(())
    }
    fn parse_kind_from_u8(&self, v: u8) -> Result<TxKind, ParserError> {
        match v {
            0 => Ok(TxKind::Deposit),
            1 => Ok(TxKind::Transfer),
            2 => Ok(TxKind::Withdrawal),
            _ => Err(ParserError::UnparsableValue(v.to_string())),
        }
    }
    fn kind_to_u8(&self, v: TxKind) -> u8 {
        match v {
            TxKind::Deposit => 0,
            TxKind::Transfer => 1,
            TxKind::Withdrawal => 2,
        }
    }

    fn parse_status_from_u8(&self, v: u8) -> Result<TxStatus, ParserError> {
        match v {
            0 => Ok(TxStatus::Success),
            1 => Ok(TxStatus::Failure),
            2 => Ok(TxStatus::Pending),
            _ => Err(ParserError::UnparsableValue(v.to_string())),
        }
    }
    fn status_to_u8(&self, v: TxStatus) -> u8 {
        match v {
            TxStatus::Success => 0,
            TxStatus::Failure => 1,
            TxStatus::Pending => 2,
        }
    }
}
impl DataParser for BinaryCodec {
    fn parse<R: Read>(&self, mut r: R) -> Result<Vec<TxRecord>, AppError> {
        let mut pos: usize = 0;
        let mut result = Vec::new();

        loop {
            // reading record signature, distinct EOF or io::Error
            let mut magic = [0u8; 4];
            match r.read_exact(&mut magic) {
                Ok(()) => pos += 4,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(AppError::ReadError(e)),
            }
            if RECORD_MAGIC != magic {
                return Err(ParserError::InvalidRecordHeader(self.bytes_to_hex(&magic)))
                    .add_parser_ctx(ParserContext::with_position(pos));
            }

            let record_size = self.read_u32_be(&mut r)?;
            pos += 4;
            if MINIMUM_RECORD_SIZE > record_size {
                return Err(ParserError::IncompleteRecord)
                    .add_parser_ctx(ParserContext::with_position(pos));
            }

            // Read record body into buffer at once
            let mut record_body = vec![0u8; record_size as usize];
            r.read_exact(&mut record_body).add_read_ctx()?;
            let mut buf = std::io::Cursor::new(record_body);

            // read and parse TXID
            let tx_id = self.read_u64_be(&mut buf)?;
            pos += 8;
            let mut b = [0u8; 1];

            // read and parse TXTYPE aka TXKIND
            buf.read_exact(&mut b).add_read_ctx()?;
            pos += 1;
            let tx_kind = self.parse_kind_from_u8(b[0]).add_parser_ctx(
                ParserContext::with_position_and_field_key(pos, TxFieldKey::TxKind),
            )?;

            // read and parse FROM
            let from = self.read_u64_be(&mut buf)?;
            pos += 8;

            // read and parse TO
            let to = self.read_u64_be(&mut buf)?;
            pos += 8;

            // read and parse AMOUNT
            let amount = self.read_i64_be(&mut buf)?;
            pos += 8;

            // read and parse TIMESTAMP
            let ts_miliseconds = self.read_u64_be(&mut buf)?;
            let ts = TxTimestamp::from_millis(ts_miliseconds);
            pos += 8;

            // read and parse STATUS
            buf.read_exact(&mut b).add_read_ctx()?;
            pos += 1;
            let status = self.parse_status_from_u8(b[0]).add_parser_ctx(
                ParserContext::with_position_and_field_key(pos, TxFieldKey::Status),
            )?;

            // read and parse DESCRIPTION
            let desc_len = self.read_u32_be(&mut buf)? as usize;
            pos += 4;
            let description = if 0 < desc_len {
                let mut desc_bytes = vec![0u8; desc_len];
                buf.read_exact(&mut desc_bytes).add_read_ctx()?;
                pos += desc_len;
                String::from_utf8(desc_bytes)
                    .map_err(|_| ParserError::UnparsableValue("non utf-8 string".into()))
                    .add_parser_ctx(ParserContext::with_position_and_field_key(
                        pos,
                        TxFieldKey::Description,
                    ))?
            } else {
                "".into()
            };

            // assemble transaction record
            result.push(TxRecord {
                id: TxIdType(tx_id),
                kind: tx_kind,
                from: AccountType(from),
                to: AccountType(to),
                amount,
                ts,
                status,
                description,
            });
        }

        Ok(result)
    }
}

impl DataWriter for BinaryCodec {
    fn write<W: Write>(&self, w: &mut W, data: &[TxRecord]) -> Result<(), AppError> {
        for rec in data {
            // pre-compute sizes
            let desc_bytes = rec.description.as_bytes();
            let record_bites = (8 + 1 + 8 + 8 + 8 + 8 + 1 + 4 + desc_bytes.len()) as u32;
            // write record header
            w.write_all(&RECORD_MAGIC).add_write_ctx()?;
            self.write_u32_be(w, record_bites)?;
            // write record body
            self.write_u64_be(w, rec.id.0)?;
            w.write_all(&[self.kind_to_u8(rec.kind)]).add_write_ctx()?;
            self.write_u64_be(w, rec.from.0)?;
            self.write_u64_be(w, rec.to.0)?;
            self.write_i64_be(w, rec.amount)?;
            self.write_u64_be(w, rec.ts.millis())?;
            w.write_all(&[self.status_to_u8(rec.status)])
                .add_write_ctx()?;
            self.write_u32_be(w, desc_bytes.len() as u32)?;
            w.write_all(desc_bytes).add_write_ctx()?;
        }
        Ok(())
    }
}
