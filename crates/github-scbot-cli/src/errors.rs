use snafu::prelude::*;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum CliError {
    #[snafu(display("{}", source))]
    ConfError {
        source: github_scbot_core::config::ConfError,
    },
    #[snafu(display("{}", source))]
    DatabaseError {
        source: github_scbot_database::DatabaseError,
    },
    #[snafu(display("{}", source))]
    UiError { source: github_scbot_tui::UiError },
    #[snafu(display("{}", source))]
    ApiError {
        source: github_scbot_ghapi::ApiError,
    },
    #[snafu(display("{}", source))]
    ServerError {
        source: github_scbot_server::ServerError,
    },
    #[snafu(display("{}", source))]
    DomainError {
        source: github_scbot_domain::DomainError,
    },
    #[snafu(display("{}", source))]
    IoError { source: std::io::Error },
    #[snafu(whatever)]
    Other { message: String },
}
