use prbot_database_interface::DatabaseError;
use prbot_models::{PullRequest, Repository, RequiredReviewer};

use crate::testcase::db_test_case;

#[tokio::test]
async fn create() {
    db_test_case("required_reviewer_create", |db| async move {
        let repo = db
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await?;

        assert!(matches!(
            db.required_reviewers_create(RequiredReviewer {
                pull_request_id: 1,
                username: "me".into(),
            })
            .await,
            Err(DatabaseError::UnknownPullRequestId(1))
        ));

        let pr = db
            .pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                ..Default::default()
            })
            .await?;

        let r = db
            .required_reviewers_create(RequiredReviewer {
                pull_request_id: pr.id,
                username: "me".into(),
            })
            .await?;

        assert_eq!(r.pull_request_id, pr.id);
        assert_eq!(r.username, "me");

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn list() {
    db_test_case("required_reviewer_list", |db| async move {
        assert_eq!(db.required_reviewers_list("me", "repo", 1).await?, vec![]);

        let repo = db
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await?;
        let pr = db
            .pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                ..Default::default()
            })
            .await?;

        let r1 = db
            .required_reviewers_create(RequiredReviewer {
                pull_request_id: pr.id,
                username: "me".into(),
            })
            .await?;
        let r2 = db
            .required_reviewers_create(RequiredReviewer {
                pull_request_id: pr.id,
                username: "her".into(),
            })
            .await?;

        assert_eq!(
            db.required_reviewers_list("owner", "name", 1).await?,
            vec![r2, r1]
        );

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn all() {
    db_test_case("required_reviewer_all", |db| async move {
        assert_eq!(db.required_reviewers_all().await?, vec![]);

        let repo = db
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await?;
        let pr1 = db
            .pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                ..Default::default()
            })
            .await?;
        let pr2 = db
            .pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 2,
                ..Default::default()
            })
            .await?;

        let r1 = db
            .required_reviewers_create(RequiredReviewer {
                pull_request_id: pr1.id,
                username: "me".into(),
            })
            .await?;
        let r2 = db
            .required_reviewers_create(RequiredReviewer {
                pull_request_id: pr1.id,
                username: "her".into(),
            })
            .await?;
        let r3 = db
            .required_reviewers_create(RequiredReviewer {
                pull_request_id: pr2.id,
                username: "me".into(),
            })
            .await?;

        assert_eq!(db.required_reviewers_all().await?, vec![r2, r1, r3]);

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn get() {
    db_test_case("required_reviewer_get", |db| async move {
        assert_eq!(
            db.required_reviewers_get("owner", "name", 1, "me").await?,
            None
        );

        let repo = db
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await?;
        let pr = db
            .pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                ..Default::default()
            })
            .await?;

        let r = db
            .required_reviewers_create(RequiredReviewer {
                pull_request_id: pr.id,
                username: "me".into(),
            })
            .await?;

        assert_eq!(
            db.required_reviewers_get("owner", "name", 1, "me").await?,
            Some(r)
        );

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn delete() {
    db_test_case("required_reviewer_delete", |db| async move {
        assert!(
            !db.required_reviewers_delete("owner", "name", 1, "me")
                .await?
        );

        let repo = db
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await?;
        let pr = db
            .pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                ..Default::default()
            })
            .await?;

        db.required_reviewers_create(RequiredReviewer {
            pull_request_id: pr.id,
            username: "me".into(),
        })
        .await?;

        assert!(
            db.required_reviewers_delete("owner", "name", 1, "me")
                .await?,
        );
        assert_eq!(
            db.required_reviewers_get("owner", "name", 1, "me").await?,
            None
        );

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn cascade_pull_request() {
    db_test_case("required_reviewer_cascade_pull_request", |db| async move {
        let repo = db
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await?;
        let pr = db
            .pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                ..Default::default()
            })
            .await?;

        db.required_reviewers_create(RequiredReviewer {
            pull_request_id: pr.id,
            username: "me".into(),
        })
        .await?;

        db.pull_requests_delete("owner", "name", 1).await?;
        assert_eq!(db.required_reviewers_all().await?, vec![]);

        Ok(())
    })
    .await;
}

#[tokio::test]
async fn cascade_repository() {
    db_test_case("required_reviewer_cascade_repository", |db| async move {
        let repo = db
            .repositories_create(Repository {
                owner: "owner".into(),
                name: "name".into(),
                ..Default::default()
            })
            .await?;
        let pr = db
            .pull_requests_create(PullRequest {
                repository_id: repo.id,
                number: 1,
                ..Default::default()
            })
            .await?;

        db.required_reviewers_create(RequiredReviewer {
            pull_request_id: pr.id,
            username: "me".into(),
        })
        .await?;

        db.repositories_delete("owner", "name").await?;
        assert_eq!(db.required_reviewers_all().await?, vec![]);

        Ok(())
    })
    .await;
}
