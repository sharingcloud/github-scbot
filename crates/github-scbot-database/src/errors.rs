use github_scbot_core::crypto::CryptoError;
use snafu::{prelude::*, Backtrace};

pub type StdError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum DatabaseError {
    #[snafu(display("Database connection error,\n  caused by: {}", source))]
    ConnectionError {
        source: sqlx::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Migration error,\n  caused by: {}", source))]
    MigrationError {
        source: sqlx::migrate::MigrateError,
        backtrace: Backtrace,
    },

    #[snafu(display("SQL error,\n  caused by: {}", source))]
    SqlError {
        source: sqlx::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Transaction error,\n  caused by: {}", source))]
    TransactionError {
        source: sqlx::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Import/Export JSON error,\n  caused by: {}", source))]
    ExchangeJsonError {
        source: serde_json::Error,
        backtrace: Backtrace,
    },

    #[snafu(display("Crypto error,\ncaused by: {}", source))]
    CryptoError {
        #[snafu(backtrace)]
        source: CryptoError,
    },
}

pub type Result<T, E = DatabaseError> = core::result::Result<T, E>;
