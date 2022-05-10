use snafu::{prelude::*, Backtrace};

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum CliError {
    #[snafu(display("{}", source))]
    ConfError {
        #[snafu(backtrace)]
        source: github_scbot_conf::ConfError,
    },
    #[snafu(display("{}", source))]
    DatabaseError {
        #[snafu(backtrace)]
        source: github_scbot_database2::DatabaseError,
    },
    #[snafu(display("{}", source))]
    UiError { source: github_scbot_tui::UiError },
    #[snafu(display("{}", source))]
    ApiError {
        #[snafu(backtrace)]
        source: github_scbot_ghapi::ApiError,
    },
    #[snafu(display("{}", source))]
    ServerError {
        source: github_scbot_server::ServerError,
    },
    #[snafu(display("{}", source))]
    LogicError {
        source: github_scbot_logic::LogicError,
    },
    #[snafu(display("{}", source))]
    IoError {
        source: std::io::Error,
        backtrace: Backtrace,
    },
    #[snafu(whatever)]
    Other {
        message: String,
        backtrace: Backtrace,
    },
}
