pub type StdError = Box<dyn std::error::Error + Send + Sync>;

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Database connection error.")]
    ConnectionError(#[source] sqlx::Error),

    #[error("Migration error.")]
    MigrationError,

    #[error("SQL error.")]
    SqlError(#[source] sqlx::Error),

    #[error("Transaction error.")]
    TransactionError(#[source] sqlx::Error),

    #[error("Import/Export error.")]
    ExchangeError(#[source] StdError),

    #[error("Unknown error.")]
    UnknownError(#[from] StdError),
}

pub type Result<T> = core::result::Result<T, DatabaseError>;
