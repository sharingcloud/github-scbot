//! Logic module.

#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

pub mod bot_commands;
mod context;
pub mod errors;
pub mod use_cases;

use bot_commands::executor::CommandExecutor;
pub use context::CoreContext;
pub use errors::{DomainError, Result};
use shaku::module;
use use_cases::{
    checks::{
        determine_check_status::DetermineChecksStatus,
        determine_commit_status::DetermineCommitStatus,
        handle_check_suite_event::HandleCheckSuiteEvent,
    },
    comments::{
        generate_random_gif_comment::GenerateRandomGifComment,
        handle_issue_comment_event::HandleIssueCommentEvent,
        post_welcome_comment::PostWelcomeComment,
    },
    gifs::random_gif_from_query::RandomGifFromQuery,
    pulls::{
        add_pull_request_rule::AddPullRequestRule, apply_pull_request_rules::ApplyPullRequestRules,
        automerge_pull_request::AutomergePullRequest,
        determine_pull_request_merge_strategy::DeterminePullRequestMergeStrategy,
        get_or_create_repository::GetOrCreateRepository, merge_pull_request::MergePullRequest,
        process_pull_request_event::ProcessPullRequestEvent,
        process_pull_request_opened::ProcessPullRequestOpened,
        remove_pull_request_rule::RemovePullRequestRule,
        resolve_pull_request_rules::ResolvePullRequestRules, set_step_label::SetStepLabel,
        synchronize_pull_request::SynchronizePullRequest,
        synchronize_pull_request_and_update_status::SynchronizePullRequestAndUpdateStatus,
        try_merge_pull_request_from_status::TryMergePullRequestFromStatus,
        update_step_label_from_status::UpdateStepLabelFromStatus,
    },
    repositories::{add_merge_rule::AddMergeRule, rename_repository::RenameRepository},
    reviews::{
        add_reviewers::AddReviewers, filter_reviewers::FilterReviewers,
        handle_review_event::HandleReviewEvent, remove_reviewers::RemoveReviewers,
    },
    status::{
        build_pull_request_status::BuildPullRequestStatus,
        create_or_update_commit_status::CreateOrUpdateCommitStatus,
        disable_pull_request_status::DisablePullRequestStatus,
        set_pull_request_qa_status::SetPullRequestQaStatus,
        update_pull_request_status::UpdatePullRequestStatus,
    },
    summary::{
        delete_summary_comment::DeleteSummaryComment, post_summary_comment::PostSummaryComment,
    },
};

module! {
    pub CoreModule {
        components = [
            AddMergeRule, DisablePullRequestStatus, MergePullRequest,
            SetStepLabel, DetermineChecksStatus, AutomergePullRequest,
            PostSummaryComment, DeleteSummaryComment, BuildPullRequestStatus,
            UpdatePullRequestStatus, RandomGifFromQuery, GenerateRandomGifComment,
            CommandExecutor, PostWelcomeComment, SynchronizePullRequest,
            ProcessPullRequestOpened, SynchronizePullRequestAndUpdateStatus,
            FilterReviewers, AddReviewers, RemoveReviewers,
            TryMergePullRequestFromStatus, DeterminePullRequestMergeStrategy,
            GetOrCreateRepository, ProcessPullRequestEvent, HandleCheckSuiteEvent,
            HandleIssueCommentEvent, HandleReviewEvent, SetPullRequestQaStatus,
            UpdateStepLabelFromStatus, CreateOrUpdateCommitStatus, RenameRepository,
            DetermineCommitStatus, ResolvePullRequestRules, ApplyPullRequestRules,
            AddPullRequestRule, RemovePullRequestRule
        ],
        providers = []
    }
}
