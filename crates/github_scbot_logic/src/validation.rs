//! Validation module.

use github_scbot_libs::regex::Regex;

use super::Result;

/// Check PR title
pub fn check_pr_title(name: &str, pattern: &str) -> Result<bool> {
    if pattern.is_empty() {
        Ok(true)
    } else {
        Regex::new(pattern)
            .map(|rgx| rgx.is_match(name))
            .map_err(Into::into)
    }
}
