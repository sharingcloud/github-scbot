//! Webhook constants.

/// GitHub event header.
pub const GITHUB_EVENT_HEADER: &str = "X-GitHub-Event";
/// GitHub signature header.
pub const GITHUB_SIGNATURE_HEADER: &str = "X-Hub-Signature-256";
/// Signature prefix length.
pub const SIGNATURE_PREFIX_LENGTH: usize = "sha256=".len();
