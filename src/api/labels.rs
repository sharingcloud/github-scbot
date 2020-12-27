//! Labels API module

use super::errors::Result;
use super::get_client;
use crate::database::models::StepLabel;

fn add_step_in_existing_labels(existing_labels: &[String], step: Option<StepLabel>) -> Vec<String> {
    let mut preserved_labels: Vec<String> = existing_labels
        .iter()
        .cloned()
        .filter(|x| StepLabel::from_str(x).is_err())
        .collect();

    if let Some(step) = step {
        preserved_labels.push(step.as_str().to_string());
    }

    preserved_labels
}

async fn get_issue_labels(
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
) -> Result<Vec<String>> {
    if cfg!(test) {
        Ok(vec![])
    } else {
        let client = get_client().await?;

        Ok(client
            .issues(repo_owner, repo_name)
            .list_labels_for_issue(pr_number)
            .send()
            .await?
            .take_items()
            .into_iter()
            .map(|x| x.name)
            .collect())
    }
}

pub async fn set_step_label(
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    label: Option<StepLabel>,
) -> Result<()> {
    if cfg!(test) {
        Ok(())
    } else {
        let existing_labels = get_issue_labels(repo_owner, repo_name, pr_number).await?;
        let existing_labels = add_step_in_existing_labels(&existing_labels, label);

        let client = get_client().await?;
        client
            .issues(repo_owner, repo_name)
            .replace_all_labels(pr_number, &existing_labels)
            .await?;

        Ok(())
    }
}
