//! Labels API module.

use std::convert::TryFrom;

use github_scbot_types::labels::StepLabel;

use crate::{adapter::IAPIAdapter, Result};

/// Add pull request step label in existing labels by returning a new vector.
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

/// Apply or remove a step label on a pull request.
pub async fn set_step_label(
    adapter: &impl IAPIAdapter,
    repository_owner: &str,
    repository_name: &str,
    pr_number: u64,
    label: Option<StepLabel>,
) -> Result<()> {
    let existing_labels = adapter
        .issue_labels_list(repository_owner, repository_name, pr_number)
        .await?;
    let existing_labels = add_step_in_existing_labels(&existing_labels, label);
    adapter
        .issue_labels_replace_all(
            repository_owner,
            repository_name,
            pr_number,
            &existing_labels,
        )
        .await?;

    Ok(())
}
