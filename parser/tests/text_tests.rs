use parser::codecs::base::Format;
use parser::codecs::base::TxFieldKey;
use parser::codecs::errors::ParserError;
use parser::errors::AppError;

const RECORD_1: &str = r#"TX_ID: 1
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 100
AMOUNT: 500
TIMESTAMP: 1700
STATUS: SUCCESS
DESCRIPTION: "Salary"
"#;

#[test]
fn parse_accepts_comments_and_two_records() {
    let input = r#"# first record
TX_ID: 1
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 100
AMOUNT: 500
TIMESTAMP: 1700
STATUS: SUCCESS
DESCRIPTION: "Salary"

# second record
TX_ID: 2
TX_TYPE: TRANSFER
FROM_USER_ID: 100
TO_USER_ID: 101
AMOUNT: 10
TIMESTAMP: 1800
STATUS: FAILURE
DESCRIPTION: "Fee"
"#;

    let records = Format::Text
        .parse(input.as_bytes())
        .expect("text parse should succeed");
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].id.0, 1);
    assert_eq!(records[1].id.0, 2);
}

#[test]
fn parse_accepts_fields_in_any_order() {
    let input = r#"DESCRIPTION: "Out of order"
STATUS: PENDING
TIMESTAMP: 1701
AMOUNT: 42
TO_USER_ID: 3
FROM_USER_ID: 2
TX_TYPE: WITHDRAWAL
TX_ID: 7
"#;

    let records = Format::Text
        .parse(input.as_bytes())
        .expect("record with shuffled fields should parse");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].id.0, 7);
    assert_eq!(records[0].description, "Out of order");
}

#[test]
fn parse_rejects_duplicate_field() {
    let input = r#"TX_ID: 1
TX_ID: 2
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 100
AMOUNT: 500
TIMESTAMP: 1700
STATUS: SUCCESS
DESCRIPTION: "Salary"
"#;

    let err = Format::Text
        .parse(input.as_bytes())
        .expect_err("duplicate key should fail");
    assert!(matches!(
        err,
        AppError::ParsingError {
            context: _,
            source: ParserError::Duplicate(TxFieldKey::Id),
        }
    ));
}

#[test]
fn parse_rejects_missing_required_field_on_record_end() {
    let input = r#"TX_ID: 1
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 100
AMOUNT: 500
TIMESTAMP: 1700
STATUS: SUCCESS

"#;

    let err = Format::Text
        .parse(input.as_bytes())
        .expect_err("missing description should fail");
    assert!(matches!(
        err,
        AppError::ParsingError {
            context: _,
            source: ParserError::MissingField(TxFieldKey::Description),
        }
    ));
}

#[test]
fn parse_rejects_line_without_delimiter() {
    let input = "TX_ID 1\n";
    let err = Format::Text
        .parse(input.as_bytes())
        .expect_err("missing delimiter should fail");
    assert!(matches!(
        err,
        AppError::ParsingError {
            context: _,
            source: ParserError::NoFieldDelimiter,
        }
    ));
}

#[test]
fn parse_rejects_unquoted_description() {
    let input = r#"TX_ID: 1
TX_TYPE: DEPOSIT
FROM_USER_ID: 0
TO_USER_ID: 100
AMOUNT: 500
TIMESTAMP: 1700
STATUS: SUCCESS
DESCRIPTION: Salary
"#;
    let err = Format::Text
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
fn parse_empty_text_input_returns_no_records() {
    let records = Format::Text
        .parse([].as_slice())
        .expect("empty text should parse");
    assert!(records.is_empty());
}

#[test]
fn text_write_then_parse_single_record() {
    let records = Format::Text
        .parse(RECORD_1.as_bytes())
        .expect("fixture should parse");

    let mut bytes = Vec::new();
    Format::Text
        .write(&mut bytes, &records)
        .expect("text write should succeed");
    let reparsed = Format::Text
        .parse(bytes.as_slice())
        .expect("written text should parse");
    assert_eq!(reparsed, records);
}
