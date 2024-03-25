use std::str::FromStr;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DriverError {
    #[error("Invalid driver kind: {kind}")]
    InvalidDriverKind { kind: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApiDriver {
    Null,
    GitHub,
}

impl FromStr for ApiDriver {
    type Err = DriverError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "null" => Ok(Self::Null),
            "github" => Ok(Self::GitHub),
            _ => Err(DriverError::InvalidDriverKind { kind: s.into() }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseDriver {
    Memory,
    Postgres,
}

impl FromStr for DatabaseDriver {
    type Err = DriverError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "memory" => Ok(Self::Memory),
            "pg" => Ok(Self::Postgres),
            _ => Err(DriverError::InvalidDriverKind { kind: s.into() }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LockDriver {
    Null,
    Redis,
}

impl FromStr for LockDriver {
    type Err = DriverError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match &s.to_lowercase()[..] {
            "null" => Ok(Self::Null),
            "redis" => Ok(Self::Redis),
            _ => panic!("Invalid driver value"),
        }
    }
}
