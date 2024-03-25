use prbot_ghapi_interface::ApiError;

#[derive(Debug, thiserror::Error)]
#[allow(clippy::enum_variant_names)]
pub enum GitHubError {
    #[error(transparent)]
    HttpError { source: reqwest::Error },

    #[error(
        "Could not merge pull request #{} on repository {}",
        pr_number,
        repository_path
    )]
    MergeError {
        pr_number: u64,
        repository_path: String,
    },

    #[error(transparent)]
    ImplementationError {
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
}

impl From<reqwest::Error> for GitHubError {
    fn from(e: reqwest::Error) -> Self {
        GitHubError::HttpError { source: e }
    }
}

impl From<GitHubError> for ApiError {
    fn from(e: GitHubError) -> Self {
        match e {
            GitHubError::MergeError {
                pr_number,
                repository_path,
            } => ApiError::MergeError {
                pr_number,
                repository_path,
            },
            e => ApiError::ImplementationError { source: e.into() },
        }
    }
}
