use crate::codecs::errors::ParserError;
use std::{
    fmt::Display,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone, Copy)]
pub struct TxIdType(pub u64);
impl Display for TxIdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Default, Eq, PartialEq, Hash, Clone, Copy)]
pub struct AccountType(pub u64);
impl Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum TxKind {
    Deposit,
    Transfer,
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

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub struct TxTimestamp(pub SystemTime);
impl TxTimestamp {
    pub fn now() -> Self {
        let t = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time can't be before the Unix epoch with ::now");
        // here we truncate nanoseconds
        let ms = t.as_millis() as u64;
        let ts = UNIX_EPOCH + Duration::from_millis(ms);
        Self(ts)
    }
    pub fn milliseconds(&self) -> u128 {
        let t = self
            .0
            .duration_since(UNIX_EPOCH)
            .expect("time can't be before the Unix");
        t.as_millis()
    }
    pub fn from_millis(milliseconds: u64) -> Option<Self> {
        UNIX_EPOCH
            .checked_add(Duration::from_millis(milliseconds))
            .map(|ts| TxTimestamp(ts))
    }
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

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum TxStatus {
    Success,
    Failure,
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

#[derive(Debug, Eq, PartialEq, Hash, Clone)]
pub struct TxRecord {
    pub id: TxIdType,
    pub kind: TxKind,
    pub from: AccountType,
    pub to: AccountType,
    pub amount: i64,
    pub ts: TxTimestamp,
    pub status: TxStatus,
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
