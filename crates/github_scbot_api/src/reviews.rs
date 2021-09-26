//! Reviews API module.

use std::collections::HashMap;

use github_scbot_types::reviews::GhReview;

use crate::{
    adapter::{GhReviewApi, IAPIAdapter},
    Result,
};

/// List reviews for pull request.
/// Dedupe reviews per reviewer (only last state is kept).
pub async fn list_reviews_for_pull_request(
    adapter: &dyn IAPIAdapter,
    repository_owner: &str,
    repository_name: &str,
    pr_number: u64,
) -> Result<Vec<GhReview>> {
    Ok(filter_last_review_states(
        adapter
            .pull_reviews_list(repository_owner, repository_name, pr_number)
            .await?,
    ))
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
    use github_scbot_libs::chrono;
    use github_scbot_types::{common::GhUser, reviews::GhReviewState};

    use super::{filter_last_review_states, GhReviewApi};
    use crate::adapter::GhReviewStateApi;

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
