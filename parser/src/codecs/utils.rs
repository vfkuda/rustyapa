use super::errors::ParserError;

// unquote description
pub(super) fn unquote<'a>(value: &'a str) -> Result<&'a str, ParserError> {
    value
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .ok_or_else(|| ParserError::ShellBeQuoted(value.into()))
}
