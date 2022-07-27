//! Repository commands.

use std::io::Write;

use crate::Result;
use async_trait::async_trait;
use clap::{Parser, Subcommand};

use super::{Command, CommandContext};

mod add;
mod list;
mod list_merge_rules;
mod remove_merge_rule;
mod set_automerge;
mod set_checks_status;
mod set_manual_interaction;
mod set_merge_rule;
mod set_qa_status;
mod set_reviewers_count;
mod set_title_regex;
mod show;

use self::{
    add::RepositoryAddCommand, list::RepositoryListCommand,
    list_merge_rules::RepositoryListMergeRulesCommand,
    remove_merge_rule::RepositoryRemoveMergeRuleCommand,
    set_automerge::RepositorySetAutomergeCommand,
    set_checks_status::RepositorySetChecksStatusCommand,
    set_manual_interaction::RepositorySetManualInteractionCommand,
    set_merge_rule::RepositorySetMergeRuleCommand, set_qa_status::RepositorySetQAStatusCommand,
    set_reviewers_count::RepositorySetReviewersCountCommand,
    set_title_regex::RepositorySetTitleRegexCommand, show::RepositoryShowCommand,
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
    SetTitleRegex(RepositorySetTitleRegexCommand),
    Show(RepositoryShowCommand),
    SetReviewersCount(RepositorySetReviewersCountCommand),
    SetMergeRule(RepositorySetMergeRuleCommand),
    SetManualInteraction(RepositorySetManualInteractionCommand),
    SetQAStatus(RepositorySetQAStatusCommand),
    SetChecksStatus(RepositorySetChecksStatusCommand),
    SetAutomerge(RepositorySetAutomergeCommand),
    RemoveMergeRule(RepositoryRemoveMergeRuleCommand),
    ListMergeRules(RepositoryListMergeRulesCommand),
    List(RepositoryListCommand),
}

#[async_trait(?Send)]
impl Command for RepositorySubCommand {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()> {
        match self {
            Self::Add(sub) => sub.execute(ctx).await,
            Self::SetTitleRegex(sub) => sub.execute(ctx).await,
            Self::Show(sub) => sub.execute(ctx).await,
            Self::SetReviewersCount(sub) => sub.execute(ctx).await,
            Self::SetMergeRule(sub) => sub.execute(ctx).await,
            Self::SetManualInteraction(sub) => sub.execute(ctx).await,
            Self::SetQAStatus(sub) => sub.execute(ctx).await,
            Self::SetChecksStatus(sub) => sub.execute(ctx).await,
            Self::SetAutomerge(sub) => sub.execute(ctx).await,
            Self::RemoveMergeRule(sub) => sub.execute(ctx).await,
            Self::ListMergeRules(sub) => sub.execute(ctx).await,
            Self::List(sub) => sub.execute(ctx).await,
        }
    }
}