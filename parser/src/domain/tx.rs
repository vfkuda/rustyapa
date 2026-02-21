use crate::codecs::errors::ParserError;
use std::{
    fmt::Display,
    time::{Duration, SystemTime, UNIX_EPOCH},
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
pub struct TxTimestamp(pub SystemTime);
impl TxTimestamp {
    /// Returns current timestamp with millisecond precision.
    pub fn now() -> Self {
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time can't be before the Unix epoch with ::now");
        // here we truncate nanoseconds
        let ms = t.as_millis() as u64;
        let ts = UNIX_EPOCH + Duration::from_millis(ms);
        Self(ts)
    }
    /// Returns milliseconds since unix epoch for this timestamp.
    pub fn milliseconds(&self) -> u128 {
        let t = self
            .0
            .duration_since(UNIX_EPOCH)
            .expect("time can't be before the Unix");
        t.as_millis()
    }
    /// Builds timestamp from UNIX epoch milliseconds.
    pub fn from_millis(milliseconds: u64) -> Option<Self> {
        UNIX_EPOCH
            .checked_add(Duration::from_millis(milliseconds))
            .map(|ts| TxTimestamp(ts))
    }
    /// Parses milliseconds since unix epoch from string.
    pub fn parse_timestamp(value: &str) -> Result<Self, ParserError> {
        let milliseconds: u64 = value.parse()?;
        TxTimestamp::from_millis(milliseconds).ok_or(ParserError::UnparsableValue(format!(
            "timestamp overflow: {}",
            value
        )))
    }
}

impl Display for TxTimestamp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.milliseconds())
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
            ts: TxTimestamp::now(),
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
        let ts = TxTimestamp::now();
        assert_eq!(ts, ts.clone());
    }
}
