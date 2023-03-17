//! Reviews API module.

use std::collections::HashMap;

use github_scbot_core::types::reviews::GhReview;

use crate::{
    review::{GhReviewApi, GhReviewStateApi},
    ApiService, Result,
};

/// Review API.
pub struct ReviewApi;

impl ReviewApi {
    /// List reviews for pull request.
    /// Dedupe reviews per reviewer (only last state is kept).
    pub async fn list_reviews_for_pull_request(
        adapter: &dyn ApiService,
        repository_owner: &str,
        repository_name: &str,
        pr_number: u64,
    ) -> Result<Vec<GhReview>> {
        Ok(Self::filter_last_review_states(
            adapter
                .pull_reviews_list(repository_owner, repository_name, pr_number)
                .await?,
        ))
    }

    fn filter_last_review_states(reviews: Vec<GhReviewApi>) -> Vec<GhReview> {
        let mut output: HashMap<String, GhReview> = HashMap::new();

        for review in reviews {
            let user_login = review.user.login.clone();
            let overwrite_review = {
                if output.contains_key(&user_login) {
                    // Comments should not replace approvals or change requests
                    !matches!(review.state, GhReviewStateApi::Commented)
                } else {
                    true
                }
            };

            if overwrite_review {
                output.insert(
                    user_login,
                    GhReview {
                        submitted_at: Some(review.submitted_at),
                        user: review.user,
                        state: review.state.into(),
                    },
                );
            }
        }

        let mut res: Vec<_> = output.into_values().collect();
        res.sort_by_key(|x| x.submitted_at);
        res
    }
}

#[cfg(test)]
mod tests {
    use github_scbot_core::{
        time::OffsetDateTime,
        types::{common::GhUser, reviews::GhReviewState},
    };

    use super::*;
    use crate::review::GhReviewStateApi;

    fn new_review(username: &str, state: GhReviewStateApi) -> GhReviewApi {
        GhReviewApi {
            state,
            submitted_at: OffsetDateTime::now_utc(),
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

        let filtered = ReviewApi::filter_last_review_states(reviews);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].state, GhReviewState::Dismissed);
        assert_eq!(&filtered[0].user.login, "test1");

        let reviews = vec![
            new_review("test1", GhReviewStateApi::Approved),
            new_review("test2", GhReviewStateApi::Approved),
            new_review("test1", GhReviewStateApi::Dismissed),
            new_review("test2", GhReviewStateApi::ChangesRequested),
        ];

        let filtered = ReviewApi::filter_last_review_states(reviews);
        assert_eq!(filtered.len(), 2);
        assert_eq!(filtered[0].state, GhReviewState::Dismissed);
        assert_eq!(&filtered[0].user.login, "test1");
        assert_eq!(filtered[1].state, GhReviewState::ChangesRequested);
        assert_eq!(&filtered[1].user.login, "test2");
    }
}
