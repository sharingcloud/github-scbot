//! Reviews API module.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use github_scbot_conf::Config;
use github_scbot_types::{
    common::GHUser,
    reviews::{GHReview, GHReviewState},
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    utils::{get_client, is_client_enabled},
    Result,
};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum GHReviewStateAPI {
    Approved,
    ChangesRequested,
    Commented,
    Dismissed,
    Pending,
}

impl From<GHReviewStateAPI> for GHReviewState {
    fn from(state_api: GHReviewStateAPI) -> Self {
        use heck::SnakeCase;

        let str_value = serde_plain::to_string(&state_api).unwrap();
        let snake_case_value = str_value.to_snake_case();
        serde_plain::from_str(&snake_case_value).unwrap()
    }
}

#[derive(Deserialize)]
struct GHReviewAPI {
    user: GHUser,
    submitted_at: DateTime<Utc>,
    state: GHReviewStateAPI,
}

/// Request reviewers for a pull request.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `repository_owner` - Repository owner
/// * `repository_name` - Repository name
/// * `pr_number` - Pull request number
/// * `reviewers` - Reviewers names
pub async fn request_reviewers_for_pull_request(
    config: &Config,
    repository_owner: &str,
    repository_name: &str,
    pr_number: u64,
    reviewers: &[String],
) -> Result<()> {
    if is_client_enabled(config) {
        let client = get_client(config).await?;
        let body = serde_json::json!({ "reviewers": reviewers });

        client
            ._post(
                client.absolute_url(format!(
                    "/repos/{}/{}/pulls/{}/requested_reviewers",
                    repository_owner, repository_name, pr_number
                ))?,
                Some(&body),
            )
            .await?;
    }

    Ok(())
}

/// Remove requested reviewers for a pull request.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `repository_owner` - Repository owner
/// * `repository_name` - Repository name
/// * `pr_number` - Pull request number
/// * `reviewers` - Reviewers names
pub async fn remove_reviewers_for_pull_request(
    config: &Config,
    repository_owner: &str,
    repository_name: &str,
    pr_number: u64,
    reviewers: &[String],
) -> Result<()> {
    if is_client_enabled(config) {
        let body = serde_json::json!({ "reviewers": reviewers });

        let client = get_client(config).await?;
        let url = client.absolute_url(format!(
            "/repos/{}/{}/pulls/{}/requested_reviewers",
            repository_owner, repository_name, pr_number
        ))?;
        let builder = client
            .request_builder(&url.into_string(), http::Method::DELETE)
            .json(&body)
            .header(http::header::ACCEPT, octocrab::format_media_type("json"));

        let response = client.execute(builder).await?;
        if response.status() != 200 {
            error!(
                "Could not remove reviewers {:?} for pull request {}/{}: status code: {}",
                reviewers,
                repository_owner,
                repository_name,
                response.status()
            );
        }
    }

    Ok(())
}

/// List reviews for pull request.
/// Dedupe reviews per reviewer (only last state is kept).
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `repository_owner` - Repository owner
/// * `repository_name` - Repository name
/// * `pr_number` - Pull request number
pub async fn list_reviews_for_pull_request(
    config: &Config,
    repository_owner: &str,
    repository_name: &str,
    pr_number: u64,
) -> Result<Vec<GHReview>> {
    if is_client_enabled(config) {
        let client = get_client(config).await?;

        let data: Vec<GHReviewAPI> = client
            .get(
                format!(
                    "/repos/{owner}/{name}/pulls/{pr_number}/reviews",
                    owner = repository_owner,
                    name = repository_name,
                    pr_number = pr_number
                ),
                None::<&()>,
            )
            .await?;

        Ok(filter_last_review_states(data))
    } else {
        Ok(vec![])
    }
}

fn filter_last_review_states(reviews: Vec<GHReviewAPI>) -> Vec<GHReview> {
    let mut output: HashMap<String, GHReview> = HashMap::new();

    for review in reviews {
        output.insert(
            review.user.login.clone(),
            GHReview {
                submitted_at: review.submitted_at,
                user: review.user,
                state: review.state.into(),
            },
        );
    }

    let mut res: Vec<_> = output.into_iter().map(|(_k, v)| v).collect();
    res.sort_by_key(|x| x.submitted_at);
    res
}

#[cfg(test)]
mod tests {
    use github_scbot_types::{common::GHUser, reviews::GHReviewState};

    use super::{filter_last_review_states, GHReviewAPI, GHReviewStateAPI};

    fn new_review(username: &str, state: GHReviewStateAPI) -> GHReviewAPI {
        GHReviewAPI {
            state,
            submitted_at: chrono::Utc::now(),
            user: GHUser {
                login: username.into(),
            },
        }
    }

    #[test]
    fn test_filter_last_review_states() {
        let reviews = vec![
            new_review("test1", GHReviewStateAPI::Commented),
            new_review("test1", GHReviewStateAPI::Approved),
            new_review("test1", GHReviewStateAPI::Dismissed),
        ];

        let filtered = filter_last_review_states(reviews);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].state, GHReviewState::Dismissed);
        assert_eq!(&filtered[0].user.login, "test1");

        let reviews = vec![
            new_review("test1", GHReviewStateAPI::Approved),
            new_review("test2", GHReviewStateAPI::Approved),
            new_review("test1", GHReviewStateAPI::Dismissed),
            new_review("test2", GHReviewStateAPI::ChangesRequested),
        ];

        let filtered = filter_last_review_states(reviews);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].state, GHReviewState::Dismissed);
        assert_eq!(&filtered[0].user.login, "test1");
        assert_eq!(filtered[1].state, GHReviewState::ChangesRequested);
        assert_eq!(&filtered[1].user.login, "test2");
    }
}
