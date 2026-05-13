use std::str::FromStr;

use derive_more::{Add, AsRef, Constructor, Display, From, Sum};
use rust_decimal::Decimal;

use super::party::PartyId;

/// A token amount with exact decimal precision
#[derive(Clone, Copy, Add, Sum, Display, From, PartialEq, Eq, PartialOrd, Ord)]
pub struct Amount(Decimal);

impl Amount {
    pub fn zero() -> Self {
        Self(Decimal::ZERO)
    }

    pub fn parse(s: &str) -> Result<Self, String> {
        Decimal::from_str(s).map(Self).map_err(|e| e.to_string())
    }
}

/// The name of a token instrument (e.g. "Amulet")
#[derive(Clone, Constructor, Display, From, AsRef, PartialEq, Eq, Hash)]
#[as_ref(str)]
pub struct InstrumentName(String);

/// Identifies a token type: admin party + instrument name
#[derive(Clone)]
pub struct InstrumentId {
    pub admin: PartyId,
    pub name: InstrumentName,
}

/// A single token holding contract
pub struct Holding {
    pub owner: PartyId,
    pub instrument: InstrumentId,
    pub amount: Amount,
    pub locked: bool,
}

/// Aggregated balance for a single token type
pub struct TokenBalance {
    pub instrument: InstrumentId,
    pub total: Amount,
    pub available: Amount,
    pub locked: Amount,
    pub holding_count: usize,
    pub locked_count: usize,
}
