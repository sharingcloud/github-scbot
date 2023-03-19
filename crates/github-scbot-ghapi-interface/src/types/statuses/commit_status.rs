/// GitHub commit status
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum GhCommitStatus {
    /// Error.
    Error,
    /// Failure.
    Failure,
    /// Pending.
    Pending,
    /// Success.
    Success,
}

impl GhCommitStatus {
    /// Convert status state to static str.
    pub fn to_str(self) -> &'static str {
        self.into()
    }
}

impl From<GhCommitStatus> for &'static str {
    fn from(status_state: GhCommitStatus) -> Self {
        match status_state {
            GhCommitStatus::Error => "error",
            GhCommitStatus::Failure => "failure",
            GhCommitStatus::Pending => "pending",
            GhCommitStatus::Success => "success",
        }
    }
}
