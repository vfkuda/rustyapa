use parser::codecs::base::Format;
use parser::codecs::errors::{ParserContext, ParserError};
use parser::domain::tx::TxRecord;
use parser::errors::AppError;

const CSV_HEADER: &str =
    "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,DESCRIPTION\n";

#[test]
fn parse_header_only_csv_returns_empty_records() {
    let parsed = Format::Csv
        .parse(CSV_HEADER.as_bytes())
        .expect("header-only csv should parse");
    assert!(parsed.is_empty());
}

#[test]
fn parse_csv_accepts_whitespace_around_fields() {
    let input = format!(
        "{}{}",
        CSV_HEADER, " 7 , DEPOSIT , 0 , 3 , 99 , 1700 , SUCCESS , \"bonus\" \n"
    );
    let parsed = Format::Csv
        .parse(input.as_bytes())
        .expect("csv with spaces should parse");
    assert_eq!(parsed.len(), 1);
    assert_eq!(parsed[0].id.0, 7);
    assert_eq!(parsed[0].description, "bonus");
}

#[test]
fn parse_rejects_invalid_header() {
    let input = "TX_ID,TX_TYPE,FROM_USER_ID,TO_USER_ID,AMOUNT,TIMESTAMP,STATUS,WRONG_COL\n1,DEPOSIT,0,1,10,11,SUCCESS,\"x\"\n";
    let err = Format::Csv
        .parse(input.as_bytes())
        .expect_err("invalid csv header should fail");
    assert!(matches!(err, AppError::ParsingError { .. }));
}

#[test]
fn parse_rejects_incomplete_record() {
    let input = format!("{}{}", CSV_HEADER, "1,DEPOSIT,0,1,10,11,SUCCESS\n");
    let err = Format::Csv
        .parse(input.as_bytes())
        .expect_err("record with missing fields should fail");
    assert!(matches!(
        err,
        AppError::ParsingError {
            context: ParserContext::LineNumAndLine {
                line_num: 1,
                line: _
            },
            source: ParserError::IncompleteRecord,
        }
    ));
}

#[test]
fn parse_rejects_unknown_tx_type() {
    let input = format!("{}{}", CSV_HEADER, "1,DEPO,0,1,10,11,SUCCESS,\"some\"\n");
    let err = Format::Csv
        .parse(input.as_bytes())
        .expect_err("unknown tx type should fail");
    assert!(matches!(
        err,
        AppError::ParsingError {
            context: _,
            source: ParserError::UnparsableValue(_),
        }
    ));
}

#[test]
fn parse_rejects_unknown_status() {
    let input = format!("{}{}", CSV_HEADER, "1,DEPOSIT,0,1,10,11,OK,\"some\"\n");
    let err = Format::Csv
        .parse(input.as_bytes())
        .expect_err("unknown status should fail");
    assert!(matches!(
        err,
        AppError::ParsingError {
            context: _,
            source: ParserError::UnparsableValue(_),
        }
    ));
}

#[test]
fn parse_rejects_unquoted_description() {
    let input = format!("{}{}", CSV_HEADER, "1,DEPOSIT,0,1,10,11,SUCCESS,some\n");
    let err = Format::Csv
        .parse(input.as_bytes())
        .expect_err("unquoted description should fail");
    assert!(matches!(
        err,
        AppError::ParsingError {
            context: _,
            source: ParserError::ShellBeQuoted(_),
        }
    ));
}

#[test]
fn csv_write_then_parse_multiple_records() {
    let input = format!(
        "{}{}{}",
        CSV_HEADER,
        "1,DEPOSIT,0,11,100,1700,SUCCESS,\"in\"\n",
        "2,WITHDRAWAL,11,0,50,1800,FAILURE,\"out\"\n"
    );
    let records = Format::Csv
        .parse(input.as_bytes())
        .expect("fixture should parse");
    assert_eq!(records.len(), 2);

    let mut out = Vec::new();
    Format::Csv
        .write(&mut out, &records)
        .expect("csv write should succeed");
    let reparsed = Format::Csv
        .parse(out.as_slice())
        .expect("written csv should parse");
    assert_eq!(reparsed, records);
}

#[test]
fn csv_format_round_trip_single_record() {
    let tx = TxRecord::default();
    let mut bytes = Vec::new();

    Format::Csv
        .write(&mut bytes, &[tx.clone()])
        .expect("csv write should succeed");

    let parsed = Format::Csv
        .parse(bytes.as_slice())
        .expect("csv parse should succeed");

    assert_eq!(parsed, vec![tx]);
}
