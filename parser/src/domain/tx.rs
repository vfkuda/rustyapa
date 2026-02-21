use crate::codecs::errors::ParserError;
use std::{
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

/// Type wrapper for transaction Id field.
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone, Copy)]
pub struct TxIdType(pub u64);
impl Display for TxIdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Type wrapper for account Id field.
#[derive(Debug, Default, Eq, PartialEq, Hash, Clone, Copy)]
pub struct AccountType(pub u64);
impl Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

/// Type wrapper for transaction operation type/kind field.
#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum TxKind {
    /// Incoming funds to destination account : 0->to.
    Deposit,
    /// Funds transfer between two accounts : from->to.
    Transfer,
    /// Outgoing funds from source account : from->0 .
    Withdrawal,
}
impl Display for TxKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxKind::Deposit => write!(f, "DEPOSIT"),
            TxKind::Transfer => write!(f, "TRANSFER"),
            TxKind::Withdrawal => write!(f, "WITHDRAWAL"),
        }
    }
}

/// Type wrapper for transaction timestamp field.
#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct TxTimestamp(pub u64);
impl TxTimestamp {
    /// Returns milliseconds for now or 0 if now before Unix epoc
    pub fn default() -> Self {
        let now_or_zero = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| d.as_millis())
            .map(|ms| ms as u64)
            .unwrap_or(0);
        Self(now_or_zero)
    }
    /// Returns timestamp as milliseconds since Unix epoc
    pub fn millis(&self) -> u64 {
        self.0
    }
    /// Create new TxTimestamp instance based on milliseconds since Unix epoch provided
    pub fn from_millis(milliseconds: u64) -> Self {
        Self(milliseconds)
    }
    /// Parses milliseconds since unix epoch from string.
    pub fn parse_timestamp(value: &str) -> Result<Self, ParserError> {
        let milliseconds: u64 = value.parse()?;
        Ok(TxTimestamp::from_millis(milliseconds))
    }
}

impl Display for TxTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.millis())
    }
}

/// Transaction processing status enum.
#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum TxStatus {
    /// Transaction was processed successfully.
    Success,
    /// Transaction porcessed with failure.
    Failure,
    /// Transaction processing is in progress.
    Pending,
}

impl Display for TxStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TxStatus::Success => write!(f, "SUCCESS"),
            TxStatus::Failure => write!(f, "FAILURE"),
            TxStatus::Pending => write!(f, "PENDING"),
        }
    }
}

/// Transaction record domain model.
#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct TxRecord {
    /// Unique transaction identifier.
    pub id: TxIdType,
    /// Transaction operation type/kind.
    pub kind: TxKind,
    /// Source account id.
    pub from: AccountType,
    /// Destination account id.
    pub to: AccountType,
    /// Amount in minimal currency units.
    pub amount: i64,
    /// Transaction processing timestamp.
    pub ts: TxTimestamp,
    /// Processing status.
    pub status: TxStatus,
    /// Transaction description/ operation purpose.
    pub description: String,
}

impl Default for TxRecord {
    fn default() -> Self {
        Self {
            id: Default::default(),
            kind: TxKind::Withdrawal,
            from: Default::default(),
            to: Default::default(),
            amount: Default::default(),
            ts: TxTimestamp::default(),
            status: TxStatus::Failure,
            description: Default::default(),
        }
    }
}

#[cfg(test)]
mod tests_tx {
    use super::*;

    #[test]
    fn tx_is_equal() {
        let tx = TxRecord::default();
        assert_eq!(tx, tx.clone());
    }

    #[test]
    fn ts_is_equal() {
        let ts = TxTimestamp(42424242);
        assert_eq!(ts, ts.clone());
    }

    #[test]
    fn ts_parse() {
        let err = TxTimestamp::parse_timestamp("asdf").expect_err("unparseable uint");
        assert!(matches!(err, ParserError::UnparsableValue { .. }));

        let err = TxTimestamp::parse_timestamp("-1").expect_err("milis before epoch");
        assert!(matches!(err, ParserError::UnparsableValue { .. }));

        let ts = TxTimestamp::parse_timestamp("42424242");
        assert!(ts.is_ok());
        assert_eq!(42424242, ts.unwrap().millis());
    }
}
