//! Comments API module

use eyre::Result;

use super::get_client;

pub async fn post_comment(
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

pub async fn update_comment(
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
