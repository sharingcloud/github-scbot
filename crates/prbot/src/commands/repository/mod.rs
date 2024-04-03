//! Repository commands.

use async_trait::async_trait;
use clap::{Parser, Subcommand};

use super::{Command, CommandContext};
use crate::Result;

mod add;
mod list;
mod merge_rule;
mod pull_request_rule;
mod rename;
mod set_default_automerge;
mod set_default_checks_status;
mod set_default_qa_status;
mod set_default_reviewers_count;
mod set_default_title_regex;
mod set_manual_interaction;
mod show;

use self::{
    add::RepositoryAddCommand, list::RepositoryListCommand, merge_rule::MergeRuleCommand,
    pull_request_rule::PullRequestRuleCommand, rename::RepositoryRenameCommand,
    set_default_automerge::RepositorySetDefaultAutomergeCommand,
    set_default_checks_status::RepositorySetDefaultChecksStatusCommand,
    set_default_qa_status::RepositorySetDefaultQaStatusCommand,
    set_default_reviewers_count::RepositorySetDefaultReviewersCountCommand,
    set_default_title_regex::RepositorySetDefaultTitleRegexCommand,
    set_manual_interaction::RepositorySetManualInteractionCommand, show::RepositoryShowCommand,
};

/// Manage repositories
#[derive(Parser)]
pub(crate) struct RepositoryCommand {
    #[clap(subcommand)]
    inner: RepositorySubCommand,
}

#[async_trait]
impl Command for RepositoryCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        self.inner.execute(ctx).await
    }
}

#[derive(Subcommand)]
enum RepositorySubCommand {
    Add(RepositoryAddCommand),
    PullRequestRules(PullRequestRuleCommand),
    MergeRules(MergeRuleCommand),
    SetDefaultTitleRegex(RepositorySetDefaultTitleRegexCommand),
    Show(RepositoryShowCommand),
    SetDefaultReviewersCount(RepositorySetDefaultReviewersCountCommand),
    SetManualInteraction(RepositorySetManualInteractionCommand),
    SetDefaultQaStatus(RepositorySetDefaultQaStatusCommand),
    SetDefaultChecksStatus(RepositorySetDefaultChecksStatusCommand),
    SetDefaultAutomerge(RepositorySetDefaultAutomergeCommand),
    Rename(RepositoryRenameCommand),
    List(RepositoryListCommand),
}

#[async_trait]
impl Command for RepositorySubCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
        match self {
            Self::Add(sub) => sub.execute(ctx).await,
            Self::SetDefaultTitleRegex(sub) => sub.execute(ctx).await,
            Self::Show(sub) => sub.execute(ctx).await,
            Self::PullRequestRules(sub) => sub.execute(ctx).await,
            Self::MergeRules(sub) => sub.execute(ctx).await,
            Self::SetDefaultReviewersCount(sub) => sub.execute(ctx).await,
            Self::SetManualInteraction(sub) => sub.execute(ctx).await,
            Self::SetDefaultQaStatus(sub) => sub.execute(ctx).await,
            Self::SetDefaultChecksStatus(sub) => sub.execute(ctx).await,
            Self::SetDefaultAutomerge(sub) => sub.execute(ctx).await,
            Self::Rename(sub) => sub.execute(ctx).await,
            Self::List(sub) => sub.execute(ctx).await,
        }
    }
}
