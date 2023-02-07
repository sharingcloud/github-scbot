//! Repository commands.

use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::{Parser, Subcommand};

use super::{Command, CommandContext};

mod add;
mod add_merge_rule;
mod list;
mod list_merge_rules;
mod remove_merge_rule;
mod set_default_automerge;
mod set_default_checks_status;
mod set_default_qa_status;
mod set_default_reviewers_count;
mod set_default_title_regex;
mod set_manual_interaction;
mod show;

use self::{
    add::RepositoryAddCommand, add_merge_rule::RepositoryAddMergeRuleCommand,
    list::RepositoryListCommand, list_merge_rules::RepositoryListMergeRulesCommand,
    remove_merge_rule::RepositoryRemoveMergeRuleCommand,
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

#[async_trait(?Send)]
impl Command for RepositoryCommand {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()> {
        self.inner.execute(ctx).await
    }
}

#[derive(Subcommand)]
enum RepositorySubCommand {
    Add(RepositoryAddCommand),
    SetDefaultTitleRegex(RepositorySetDefaultTitleRegexCommand),
    Show(RepositoryShowCommand),
    SetDefaultReviewersCount(RepositorySetDefaultReviewersCountCommand),
    AddMergeRule(RepositoryAddMergeRuleCommand),
    SetManualInteraction(RepositorySetManualInteractionCommand),
    SetDefaultQaStatus(RepositorySetDefaultQaStatusCommand),
    SetDefaultChecksStatus(RepositorySetDefaultChecksStatusCommand),
    SetDefaultAutomerge(RepositorySetDefaultAutomergeCommand),
    RemoveMergeRule(RepositoryRemoveMergeRuleCommand),
    ListMergeRules(RepositoryListMergeRulesCommand),
    List(RepositoryListCommand),
}

#[async_trait(?Send)]
impl Command for RepositorySubCommand {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()> {
        match self {
            Self::Add(sub) => sub.execute(ctx).await,
            Self::SetDefaultTitleRegex(sub) => sub.execute(ctx).await,
            Self::Show(sub) => sub.execute(ctx).await,
            Self::SetDefaultReviewersCount(sub) => sub.execute(ctx).await,
            Self::AddMergeRule(sub) => sub.execute(ctx).await,
            Self::SetManualInteraction(sub) => sub.execute(ctx).await,
            Self::SetDefaultQaStatus(sub) => sub.execute(ctx).await,
            Self::SetDefaultChecksStatus(sub) => sub.execute(ctx).await,
            Self::SetDefaultAutomerge(sub) => sub.execute(ctx).await,
            Self::RemoveMergeRule(sub) => sub.execute(ctx).await,
            Self::ListMergeRules(sub) => sub.execute(ctx).await,
            Self::List(sub) => sub.execute(ctx).await,
        }
    }
}
