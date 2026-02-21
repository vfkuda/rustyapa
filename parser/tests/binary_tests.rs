use parser::codecs::base::Codec;
use parser::codecs::errors::ParserError;
use parser::domain::tx::{AccountType, TxIdType, TxKind, TxRecord, TxStatus, TxTimestamp};
use parser::errors::AppError;

fn sample_tx() -> TxRecord {
    TxRecord {
        id: TxIdType(1),
        kind: TxKind::Transfer,
        from: AccountType(11),
        to: AccountType(22),
        amount: -500,
        ts: TxTimestamp::from_millis(1_700_000),
        status: TxStatus::Pending,
        description: "payment".to_string(),
    }
}

fn write_u32_be(buf: &mut Vec<u8>, n: u32) {
    buf.extend_from_slice(&n.to_be_bytes());
}

fn write_u64_be(buf: &mut Vec<u8>, n: u64) {
    buf.extend_from_slice(&n.to_be_bytes());
}

fn write_i64_be(buf: &mut Vec<u8>, n: i64) {
    buf.extend_from_slice(&n.to_be_bytes());
}

fn encode_record(
    kind: u8,
    status: u8,
    desc: &[u8],
    override_record_size: Option<u32>,
    magic: [u8; 4],
) -> Vec<u8> {
    let mut body = Vec::new();
    write_u64_be(&mut body, 1);
    body.push(kind);
    write_u64_be(&mut body, 10);
    write_u64_be(&mut body, 20);
    write_i64_be(&mut body, 100);
    write_u64_be(&mut body, 1234);
    body.push(status);
    write_u32_be(&mut body, desc.len() as u32);
    body.extend_from_slice(desc);

    let mut out = Vec::new();
    out.extend_from_slice(&magic);
    write_u32_be(&mut out, override_record_size.unwrap_or(body.len() as u32));
    out.extend_from_slice(&body);
    out
}

#[test]
fn parse_empty_binary_is_ok_and_returns_no_records() {
    let parsed = Codec::BinaryCodec
        .parse([].as_slice())
        .expect("empty binary stream should parse");
    assert!(parsed.is_empty());
}

#[test]
fn binary_round_trip_multiple_records() {
    let tx1 = sample_tx();
    let mut tx2 = sample_tx();
    tx2.id = TxIdType(2);
    tx2.status = TxStatus::Success;
    tx2.description = "refund".to_string();

    let mut bytes = Vec::new();
    Codec::BinaryCodec
        .write(&mut bytes, &[tx1.clone(), tx2.clone()])
        .expect("binary write should succeed");

    let parsed = Codec::BinaryCodec
        .parse(bytes.as_slice())
        .expect("binary parse should succeed");
    assert_eq!(parsed, vec![tx1, tx2]);
}

#[test]
fn parse_rejects_invalid_magic_header() {
    let input = encode_record(0, 0, b"ok", None, *b"NOPE");
    let err = Codec::BinaryCodec
        .parse(input.as_slice())
        .expect_err("invalid magic should fail");
    assert!(matches!(
        err,
        AppError::ParsingError {
            context: _,
            source: ParserError::InvalidRecordHeader(_)
        }
    ));
}

#[test]
fn parse_rejects_too_small_record_size() {
    let input = encode_record(0, 0, b"", Some(45), *b"YPBN");
    let err = Codec::BinaryCodec
        .parse(input.as_slice())
        .expect_err("too small record size should fail");
    assert!(matches!(
        err,
        AppError::ParsingError {
            context: _,
            source: ParserError::IncompleteRecord
        }
    ));
}

#[test]
fn parse_rejects_unknown_kind_value() {
    let input = encode_record(9, 0, b"ok", None, *b"YPBN");
    let err = Codec::BinaryCodec
        .parse(input.as_slice())
        .expect_err("unknown tx kind should fail");
    assert!(matches!(
        err,
        AppError::ParsingError {
            context: _,
            source: ParserError::UnparsableValue(_)
        }
    ));
}

#[test]
fn parse_rejects_unknown_status_value() {
    let input = encode_record(0, 9, b"ok", None, *b"YPBN");
    let err = Codec::BinaryCodec
        .parse(input.as_slice())
        .expect_err("unknown tx status should fail");
    assert!(matches!(
        err,
        AppError::ParsingError {
            context: _,
            source: ParserError::UnparsableValue(_)
        }
    ));
}

#[test]
fn parse_rejects_non_utf8_description() {
    let input = encode_record(0, 0, &[0xFF, 0xFF], None, *b"YPBN");
    let err = Codec::BinaryCodec
        .parse(input.as_slice())
        .expect_err("invalid UTF-8 description should fail");
    assert!(matches!(
        err,
        AppError::ParsingError {
            context: _,
            source: ParserError::UnparsableValue(_)
        }
    ));
}

#[test]
fn parse_returns_read_error_for_truncated_body() {
    let mut input = encode_record(0, 0, b"ok", None, *b"YPBN");
    input.truncate(input.len() - 2);
    let err = Codec::BinaryCodec
        .parse(input.as_slice())
        .expect_err("truncated body should fail");
    assert!(matches!(err, AppError::ReadError(_)));
}
