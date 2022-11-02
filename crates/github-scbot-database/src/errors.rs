use github_scbot_core::crypto::CryptoError;

use thiserror::Error;

pub type StdError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Database connection error,\n  caused by: {}", source)]
    ConnectionError { source: sqlx::Error },

    #[error("Migration error,\n  caused by: {}", source)]
    MigrationError { source: sqlx::migrate::MigrateError },

    #[error("SQL error,\n  caused by: {}", source)]
    SqlError { source: sqlx::Error },

    #[error("Transaction error,\n  caused by: {}", source)]
    TransactionError { source: sqlx::Error },

    #[error("Import/Export JSON error,\n  caused by: {}", source)]
    ExchangeJsonError { source: serde_json::Error },

    #[error("Crypto error,\ncaused by: {}", source)]
    CryptoError { source: CryptoError },
}

pub type Result<T, E = DatabaseError> = core::result::Result<T, E>;
