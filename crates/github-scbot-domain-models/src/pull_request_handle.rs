use crate::RepositoryPath;

#[derive(Debug, Clone)]
pub struct PullRequestHandle {
    repository: RepositoryPath,
    number: u64,
}

impl std::fmt::Display for PullRequestHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{} (#{})", self.repository, self.number))
    }
}

impl PullRequestHandle {
    pub fn new(repository: RepositoryPath, number: u64) -> Self {
        Self { repository, number }
    }

    pub fn repository(&self) -> &RepositoryPath {
        &self.repository
    }

    pub fn number(&self) -> u64 {
        self.number
    }
}

impl From<(&str, &str, u64)> for PullRequestHandle {
    fn from((owner, name, number): (&str, &str, u64)) -> Self {
        Self {
            repository: (owner, name).into(),
            number,
        }
    }
}
