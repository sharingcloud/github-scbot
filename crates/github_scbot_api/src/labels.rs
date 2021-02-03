//! Labels API module.

use std::convert::TryFrom;

use github_scbot_types::labels::StepLabel;

use super::{errors::Result, get_client, is_client_enabled};

/// Add pull request step label in existing labels by returning a new vector.
///
/// # Arguments
///
/// * `existing_labels` - Existing labels
/// * `step` - Optional step label
pub fn add_step_in_existing_labels(
    existing_labels: &[String],
    step: Option<StepLabel>,
) -> Vec<String> {
    let mut preserved_labels: Vec<String> = existing_labels
        .iter()
        .cloned()
        .filter(|x| StepLabel::try_from(&x[..]).is_err())
        .collect();

    if let Some(step) = step {
        preserved_labels.push(step.to_str().to_string());
    }

    preserved_labels
}

/// Get existing labels from an issue.
///
/// # Arguments
///
/// * `repository_owner` - Repository owner
/// * `repository_name` - Repository name
/// * `issue_number` - Issue number
pub async fn get_issue_labels(
    repository_owner: &str,
    repository_name: &str,
    issue_number: u64,
) -> Result<Vec<String>> {
    if is_client_enabled() {
        let client = get_client().await?;

        Ok(client
            .issues(repository_owner, repository_name)
            .list_labels_for_issue(issue_number)
            .send()
            .await?
            .take_items()
            .into_iter()
            .map(|x| x.name)
            .collect())
    } else {
        Ok(vec![])
    }
}

/// Apply or remove a step label on a pull request.
///
/// # Arguments
///
/// * `repository_owner` - Repository owner
/// * `repository_name` - Repository name
/// * `pr_number` - Pull request number
/// * `label` - Optional step label
pub async fn set_step_label(
    repository_owner: &str,
    repository_name: &str,
    pr_number: u64,
    label: Option<StepLabel>,
) -> Result<()> {
    if is_client_enabled() {
        let existing_labels =
            get_issue_labels(repository_owner, repository_name, pr_number).await?;
        let existing_labels = add_step_in_existing_labels(&existing_labels, label);

        let client = get_client().await?;
        client
            .issues(repository_owner, repository_name)
            .replace_all_labels(pr_number, &existing_labels)
            .await?;

        Ok(())
    } else {
        Ok(())
    }
}
