//! Reviews API module.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use github_scbot_conf::Config;
use github_scbot_types::{
    common::GhUser,
    reviews::{GhReview, GhReviewState},
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{
    utils::{get_client, is_client_enabled},
    Result,
};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
enum GhReviewStateApi {
    Approved,
    ChangesRequested,
    Commented,
    Dismissed,
    Pending,
}

impl From<GhReviewStateApi> for GhReviewState {
    fn from(state_api: GhReviewStateApi) -> Self {
        use heck::SnakeCase;

        let str_value = serde_plain::to_string(&state_api).unwrap();
        let snake_case_value = str_value.to_snake_case();
        serde_plain::from_str(&snake_case_value).unwrap()
    }
}

#[derive(Deserialize)]
struct GhReviewApi {
    user: GhUser,
    submitted_at: DateTime<Utc>,
    state: GhReviewStateApi,
}

/// Request reviewers for a pull request.
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
                reviewers = ?reviewers,
                repository_path = %format!("{}/{}", repository_owner, repository_name),
                status_code = %response.status(),
                message = "Could not remove reviewers",
            );
        }
    }

    Ok(())
}

/// List reviews for pull request.
/// Dedupe reviews per reviewer (only last state is kept).
pub async fn list_reviews_for_pull_request(
    config: &Config,
    repository_owner: &str,
    repository_name: &str,
    pr_number: u64,
) -> Result<Vec<GhReview>> {
    if is_client_enabled(config) {
        let client = get_client(config).await?;

        let data: Vec<GhReviewApi> = client
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

fn filter_last_review_states(reviews: Vec<GhReviewApi>) -> Vec<GhReview> {
    let mut output: HashMap<String, GhReview> = HashMap::new();

    for review in reviews {
        output.insert(
            review.user.login.clone(),
            GhReview {
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
    use github_scbot_types::{common::GhUser, reviews::GhReviewState};

    use super::{filter_last_review_states, GhReviewApi, GhReviewStateApi};

    fn new_review(username: &str, state: GhReviewStateApi) -> GhReviewApi {
        GhReviewApi {
            state,
            submitted_at: chrono::Utc::now(),
            user: GhUser {
                login: username.into(),
            },
        }
    }

    #[test]
    fn test_filter_last_review_states() {
        let reviews = vec![
            new_review("test1", GhReviewStateApi::Commented),
            new_review("test1", GhReviewStateApi::Approved),
            new_review("test1", GhReviewStateApi::Dismissed),
        ];

        let filtered = filter_last_review_states(reviews);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].state, GhReviewState::Dismissed);
        assert_eq!(&filtered[0].user.login, "test1");

        let reviews = vec![
            new_review("test1", GhReviewStateApi::Approved),
            new_review("test2", GhReviewStateApi::Approved),
            new_review("test1", GhReviewStateApi::Dismissed),
            new_review("test2", GhReviewStateApi::ChangesRequested),
        ];

        let filtered = filter_last_review_states(reviews);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].state, GhReviewState::Dismissed);
        assert_eq!(&filtered[0].user.login, "test1");
        assert_eq!(filtered[1].state, GhReviewState::ChangesRequested);
        assert_eq!(&filtered[1].user.login, "test2");
    }
}
