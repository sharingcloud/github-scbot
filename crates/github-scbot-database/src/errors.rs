use github_scbot_core::crypto::CryptoError;
use snafu::prelude::*;

pub type StdError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum DatabaseError {
    #[snafu(display("Database connection error,\n  caused by: {}", source))]
    ConnectionError { source: sqlx::Error },

    #[snafu(display("Migration error,\n  caused by: {}", source))]
    MigrationError { source: sqlx::migrate::MigrateError },

    #[snafu(display("SQL error,\n  caused by: {}", source))]
    SqlError { source: sqlx::Error },

    #[snafu(display("Transaction error,\n  caused by: {}", source))]
    TransactionError { source: sqlx::Error },

    #[snafu(display("Import/Export JSON error,\n  caused by: {}", source))]
    ExchangeJsonError { source: serde_json::Error },

    #[snafu(display("Crypto error,\ncaused by: {}", source))]
    CryptoError { source: CryptoError },
}

pub type Result<T, E = DatabaseError> = core::result::Result<T, E>;
