//! Comments API module

use eyre::Result;

use super::constants::ENV_DISABLE_WELCOME_COMMENTS;
use super::get_client;

async fn post_comment(
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    body: &str,
) -> Result<u64> {
    if cfg!(test) {
        // No comment
        Ok(0)
    } else {
        let client = get_client().await?;

        let final_body = format!("{}\n\n_Beep boop, i'm a bot!_ :robot:", body);

        let comment = client
            .issues(repo_owner, repo_name)
            .create_comment(pr_number, final_body)
            .await?;

        Ok(comment.id)
    }
}

async fn update_comment(
    repo_owner: &str,
    repo_name: &str,
    comment_id: u64,
    body: &str,
) -> Result<u64> {
    if !cfg!(test) {
        let client = get_client().await?;

        client
            .issues(repo_owner, repo_name)
            .update_comment(comment_id, body)
            .await?;
    }

    Ok(comment_id)
}

pub async fn post_welcome_comment(
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    pr_author: &str,
) -> Result<()> {
    if std::env::var(ENV_DISABLE_WELCOME_COMMENTS).ok().is_none() {
        post_comment(
            repo_owner,
            repo_name,
            pr_number,
            &format!(
                ":tada: Welcome, _{}_ ! :tada:\n\
            Thanks for your pull request, it will be reviewed soon. :clock2:",
                pr_author
            ),
        )
        .await?;
    }

    Ok(())
}

pub async fn post_check_suite_failure_comment(
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    pr_author: &str,
) -> Result<u64> {
    post_comment(
        repo_owner,
        repo_name,
        pr_number,
        &format!(
            ":boom: Check suite failed for your PR, @{}. :boom:\n\
        You can check run logs to help you fix everything.",
            pr_author
        ),
    )
    .await
}

pub async fn post_check_suite_success_comment(
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    pr_author: &str,
) -> Result<u64> {
    post_comment(
        repo_owner,
        repo_name,
        pr_number,
        &format!(
            ":tada: Check suite run successfully for your PR, @{}. :tada:\n\
        Ready for review.",
            pr_author
        ),
    )
    .await
}

pub async fn create_or_update_status_comment(
    repo_owner: &str,
    repo_name: &str,
    pr_number: u64,
    comment_id: u64,
) -> Result<u64> {
    let status_comment = format!(
        "**Status comment**\n\
        - [{}] Checks passed\n\
        - [{}] Code reviews passed\n\
        - [{}] QA passed\n",
        " ", "x", "x"
    );

    if comment_id > 0 {
        update_comment(repo_owner, repo_name, comment_id, &status_comment).await
    } else {
        post_comment(repo_owner, repo_name, pr_number, &status_comment).await
    }
}
