//! Commands module.

use github_scbot_api::{
    comments::{add_reaction_to_comment, post_comment},
    pulls::{get_pull_request_sha, merge_pull_request},
    reviews::{remove_reviewers_for_pull_request, request_reviewers_for_pull_request},
};
use github_scbot_conf::Config;
use github_scbot_database::{
    get_connection,
    models::{PullRequestModel, RepositoryModel, ReviewModel},
    DbConn, DbPool,
};
use github_scbot_types::{issues::GhReactionType, labels::StepLabel, status::QaStatus};
use smart_default::SmartDefault;
use tracing::info;

use super::{errors::Result, status::update_pull_request_status};
use crate::{
    auth::{has_right_on_pull_request, is_admin, list_known_admin_usernames},
    gif::generate_random_gif_comment,
    pulls::{determine_automatic_step, get_merge_strategy_for_branches, synchronize_pull_request},
};

/// Command handling status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, SmartDefault)]
pub enum CommandHandlingStatus {
    /// Command handled.
    #[default]
    Handled,
    /// Command denied.
    Denied,
    /// Command ignored.
    Ignored,
}

/// Command.
#[derive(Debug, PartialEq)]
pub enum Command {
    /// Skip QA status.
    SkipQaStatus(bool),
    /// Enable/Disable QA status.
    QaStatus(Option<bool>),
    /// Enable/Disable automerge.
    Automerge(bool),
    /// Assign required reviewers.
    AssignRequiredReviewers(Vec<String>),
    /// Unassign required reviewers.
    UnassignRequiredReviewers(Vec<String>),
    /// Add/Remove lock with optional reason.
    Lock(bool, Option<String>),
    /// Post a random gif.
    Gif(String),
    /// Merge pull request.
    Merge,
    /// Ping the bot.
    Ping,
    /// Show help message.
    Help,
    /// Show admin help message.
    AdminHelp,
    /// Synchronize status.
    AdminSynchronize,
    /// Enable bot on pull request (used with manual interaction).
    AdminEnable,
    /// Set needed reviewers count.
    AdminSetNeededReviewers(u32),
}

/// Command execution result.
#[derive(Debug, PartialEq)]
pub struct CommandExecutionResult {
    /// Should update status.
    pub should_update_status: bool,
    /// Handling status.
    pub handling_status: CommandHandlingStatus,
    /// Actions.
    pub result_actions: Vec<ResultAction>,
}

impl CommandExecutionResult {
    /// Create builder instance.
    pub fn builder() -> CommandExecutionResultBuilder {
        CommandExecutionResultBuilder::default()
    }
}

/// Command execution result builder.
#[derive(Debug, Default)]
pub struct CommandExecutionResultBuilder {
    should_update_status: bool,
    handling_status: CommandHandlingStatus,
    result_actions: Vec<ResultAction>,
}

impl CommandExecutionResultBuilder {
    /// Set status update.
    pub fn with_status_update(mut self, value: bool) -> Self {
        self.should_update_status = value;
        self
    }

    /// Set ignored result.
    pub fn ignored(mut self) -> Self {
        self.handling_status = CommandHandlingStatus::Ignored;
        self
    }

    /// Set denied result.
    pub fn denied(mut self) -> Self {
        self.handling_status = CommandHandlingStatus::Denied;
        self
    }

    /// Set handled result.
    pub fn handled(mut self) -> Self {
        self.handling_status = CommandHandlingStatus::Handled;
        self
    }

    /// Add result action.
    pub fn with_action(mut self, action: ResultAction) -> Self {
        self.result_actions.push(action);
        self
    }

    /// Add multiple result actions.
    pub fn with_actions(mut self, actions: Vec<ResultAction>) -> Self {
        self.result_actions.extend(actions);
        self
    }

    /// Build execution result.
    pub fn build(self) -> CommandExecutionResult {
        CommandExecutionResult {
            handling_status: self.handling_status,
            result_actions: self.result_actions,
            should_update_status: self.should_update_status,
        }
    }
}

/// Result action.
#[derive(Debug, PartialEq)]
pub enum ResultAction {
    /// Post comment.
    PostComment(String),
    /// Add reaction.
    AddReaction(GhReactionType),
}

impl Command {
    /// Create a command from a comment and arguments.
    pub fn from_comment(comment: &str, args: &[&str]) -> Option<Self> {
        Some(match comment {
            "noqa+" => Self::SkipQaStatus(true),
            "noqa-" => Self::SkipQaStatus(false),
            "qa+" => Self::QaStatus(Some(true)),
            "qa-" => Self::QaStatus(Some(false)),
            "qa?" => Self::QaStatus(None),
            "automerge+" => Self::Automerge(true),
            "automerge-" => Self::Automerge(false),
            "lock+" => Self::Lock(true, Self::parse_message(args)),
            "lock-" => Self::Lock(false, Self::parse_message(args)),
            "req+" => Self::AssignRequiredReviewers(Self::parse_reviewers(args)),
            "req-" => Self::UnassignRequiredReviewers(Self::parse_reviewers(args)),
            "gif" => Self::Gif(Self::parse_text(args)),
            "merge" => Self::Merge,
            "ping" => Self::Ping,
            "help" => Self::Help,
            // Admin commands
            "admin-sync" => Self::AdminSynchronize,
            "admin-help" => Self::AdminHelp,
            "admin-enable" => Self::AdminEnable,
            "admin-set-needed-reviewers" => Self::AdminSetNeededReviewers(Self::parse_u32(args)),
            _ => return None,
        })
    }

    fn to_command_string(&self) -> String {
        match self {
            Self::AdminEnable => "admin-enable".into(),
            Self::AdminHelp => "admin-help".into(),
            Self::AdminSetNeededReviewers(count) => format!("admin-set-needed-reviewers {}", count),
            Self::AdminSynchronize => "admin-sync".into(),
            Self::AssignRequiredReviewers(reviewers) => format!("req+ {}", reviewers.join(" ")),
            Self::Automerge(status) => format!("automerge{}", if *status { "+" } else { "-" }),
            Self::Gif(search) => format!("gif {}", search),
            Self::Help => "help".into(),
            Self::Lock(status, reason) => {
                let mut lock = format!("lock{}", if *status { "+" } else { "-" });
                if let Some(reason) = reason {
                    lock = format!("{} {}", lock, reason);
                }
                lock
            }
            Self::Merge => "merge".into(),
            Self::Ping => "ping".into(),
            Self::QaStatus(status) => format!(
                "qa{}",
                match status {
                    None => "?",
                    Some(true) => "+",
                    Some(false) => "-",
                }
            ),
            Self::SkipQaStatus(status) => format!("noqa{}", if *status { "+" } else { "-" }),
            Self::UnassignRequiredReviewers(reviewers) => format!("req- {}", reviewers.join(" ")),
        }
    }

    fn parse_u32(args: &[&str]) -> u32 {
        args.join(" ").parse().unwrap_or(2)
    }

    fn parse_message(args: &[&str]) -> Option<String> {
        if args.is_empty() {
            None
        } else {
            Some(args.join(" "))
        }
    }

    fn parse_text(words: &[&str]) -> String {
        words.join(" ")
    }

    fn parse_reviewers(reviewers: &[&str]) -> Vec<String> {
        reviewers
            .iter()
            .filter_map(|x| x.strip_prefix('@').map(str::to_string))
            .collect()
    }

    /// Convert to bot string.
    pub fn to_bot_string(&self, config: &Config) -> String {
        format!(
            "{bot} {command}",
            bot = config.bot_username,
            command = self.to_command_string()
        )
    }
}

/// Parse commands from comment body.
pub fn parse_commands(config: &Config, comment_body: &str) -> Result<Vec<Command>> {
    let mut commands = vec![];

    for line in comment_body.lines() {
        if let Some(command) = parse_single_command(config, line)? {
            commands.push(command);
        }
    }

    Ok(commands)
}

/// Execute multiple commands.
pub async fn execute_commands(
    config: &Config,
    pool: DbPool,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_id: u64,
    comment_author: &str,
    commands: Vec<Command>,
) -> Result<()> {
    let mut status = vec![];

    for command in commands {
        status.push(
            execute_command(
                config,
                pool.clone(),
                repo_model,
                pr_model,
                comment_author,
                command,
            )
            .await?,
        );
    }

    // Merge and handle command result
    let command_result = merge_command_results(status);
    process_command_result(
        config,
        pool.clone(),
        repo_model,
        pr_model,
        comment_id,
        command_result,
    )
    .await?;

    Ok(())
}

/// Process command result.
pub async fn process_command_result(
    config: &Config,
    pool: DbPool,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_id: u64,
    command_result: CommandExecutionResult,
) -> Result<()> {
    if command_result.should_update_status {
        let sha = get_pull_request_sha(
            config,
            &repo_model.owner,
            &repo_model.name,
            pr_model.get_number(),
        )
        .await?;
        update_pull_request_status(config, pool.clone(), repo_model, pr_model, &sha).await?;
    }

    for action in command_result.result_actions {
        match action {
            ResultAction::AddReaction(reaction) => {
                add_reaction_to_comment(
                    config,
                    &repo_model.owner,
                    &repo_model.name,
                    comment_id,
                    reaction,
                )
                .await?;
            }
            ResultAction::PostComment(comment) => {
                post_comment(
                    config,
                    &repo_model.owner,
                    &repo_model.name,
                    pr_model.get_number(),
                    &comment,
                )
                .await?;
            }
        }
    }

    Ok(())
}

/// Merge command results.
pub fn merge_command_results(results: Vec<CommandExecutionResult>) -> CommandExecutionResult {
    let mut handling_status = CommandHandlingStatus::Ignored;
    let mut result_actions = vec![];
    let mut should_update_status = false;

    for result in results {
        use CommandHandlingStatus::*;

        handling_status = match (handling_status, result.handling_status) {
            (Ignored, Denied) => Denied,
            (Denied, Denied) => Denied,
            (_, Handled) => Handled,
            (Handled, _) => Handled,
            (previous, Ignored) => previous,
        };

        should_update_status = match (should_update_status, result.should_update_status) {
            (_, true) => true,
            (true, _) => true,
            (false, false) => false,
        };

        result_actions.extend(result.result_actions);
    }

    // Merge actions
    let mut merged_actions = vec![];
    let mut comments = vec![];
    for action in result_actions {
        // If action already present, ignores
        if merged_actions.contains(&action) {
            continue;
        }

        if let ResultAction::PostComment(comment) = action {
            comments.push(comment);
        } else {
            merged_actions.push(action);
        }
    }

    // Create only one comment action
    if !comments.is_empty() {
        merged_actions.push(ResultAction::PostComment(comments.join("\n\n---\n\n")));
    }

    CommandExecutionResult {
        handling_status,
        result_actions: merged_actions,
        should_update_status,
    }
}

/// Execute command.
pub async fn execute_command(
    config: &Config,
    pool: DbPool,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
    command: Command,
) -> Result<CommandExecutionResult> {
    let conn = get_connection(&pool)?;
    let mut command_result: CommandExecutionResult;

    info!(
        command = ?command,
        comment_author = comment_author,
        repository_path = %repo_model.get_path(),
        pull_request_number = pr_model.get_number(),
        message = "Interpreting command"
    );

    if !validate_user_rights_on_command(&conn, comment_author, pr_model, &command)? {
        command_result = CommandExecutionResult::builder()
            .denied()
            .with_action(ResultAction::AddReaction(GhReactionType::MinusOne))
            .build();
    } else {
        command_result = match &command {
            Command::Automerge(s) => {
                handle_auto_merge_command(&conn, pr_model, comment_author, *s).await?
            }
            Command::SkipQaStatus(s) => handle_skip_qa_command(&conn, pr_model, *s)?,
            Command::QaStatus(s) => handle_qa_command(&conn, pr_model, comment_author, *s).await?,
            Command::Lock(s, reason) => {
                handle_lock_command(&conn, pr_model, comment_author, *s, reason.clone()).await?
            }
            Command::Ping => handle_ping_command(comment_author).await?,
            Command::Merge => {
                handle_merge_command(config, &conn, repo_model, pr_model, comment_author).await?
            }
            Command::AssignRequiredReviewers(reviewers) => {
                handle_assign_required_reviewers_command(
                    config,
                    &conn,
                    repo_model,
                    pr_model,
                    reviewers.clone(),
                )
                .await?
            }
            Command::UnassignRequiredReviewers(reviewers) => {
                handle_unassign_required_reviewers_command(
                    config,
                    &conn,
                    repo_model,
                    pr_model,
                    reviewers.clone(),
                )
                .await?
            }
            Command::Gif(terms) => handle_gif_command(config, &terms).await?,
            Command::Help => handle_help_command(config, comment_author).await?,
            Command::AdminHelp => handle_admin_help_command(config, comment_author).await?,
            Command::AdminEnable => CommandExecutionResult::builder().ignored().build(),
            Command::AdminSynchronize => {
                handle_admin_sync_command(config, &conn, repo_model, pr_model).await?
            }
            Command::AdminSetNeededReviewers(count) => {
                handle_set_needed_reviewers_command(&conn, pr_model, *count).await?
            }
        };

        for action in &mut command_result.result_actions {
            if let ResultAction::PostComment(comment) = action {
                // Include command recap before comment
                *comment = format!("> {}\n\n{}", command.to_bot_string(config), comment);
            }
        }
    }

    Ok(command_result)
}

/// Parse command from a single comment line.
pub fn parse_single_command(config: &Config, line: &str) -> Result<Option<Command>> {
    if let Some((command_line, args)) = parse_command_string_from_comment_line(config, line) {
        let command = Command::from_comment(command_line, &args);
        Ok(command)
    } else {
        Ok(None)
    }
}

/// Validate user rights on command.
pub fn validate_user_rights_on_command(
    conn: &DbConn,
    username: &str,
    pr_model: &PullRequestModel,
    command: &Command,
) -> Result<bool> {
    let known_admins = list_known_admin_usernames(conn)?;

    match command {
        Command::Ping | Command::Help | Command::Gif(_) => Ok(true),
        Command::AdminEnable | Command::AdminHelp | Command::AdminSynchronize => {
            Ok(is_admin(username, &known_admins))
        }
        _ => Ok(has_right_on_pull_request(username, pr_model, &known_admins)),
    }
}

/// Parse command string from comment line.
pub fn parse_command_string_from_comment_line<'a>(
    config: &Config,
    comment: &'a str,
) -> Option<(&'a str, Vec<&'a str>)> {
    if comment.starts_with(&config.bot_username) {
        // Plus one for the '@' symbol
        let (_, command) = comment.split_at(config.bot_username.len());
        let mut split = command.trim().split_whitespace();

        if let Some(command) = split.next() {
            // Take command and remaining args
            return Some((command, split.collect()));
        }
    }

    None
}

/// Handle `Automerge` command.
pub async fn handle_auto_merge_command(
    conn: &DbConn,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
    status: bool,
) -> Result<CommandExecutionResult> {
    pr_model.automerge = status;
    pr_model.save(conn)?;

    let status_text = if status { "enabled" } else { "disabled" };
    let comment = format!("Automerge {} by **{}**", status_text, comment_author);
    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::PostComment(comment))
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .build())
}

/// Handle `Merge` command.
pub async fn handle_merge_command(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
) -> Result<CommandExecutionResult> {
    // Use step to determine merge possibility
    let reviews = pr_model.get_reviews(conn)?;
    let step = determine_automatic_step(repo_model, pr_model, &reviews)?;
    let commit_title = pr_model.get_merge_commit_title();
    let strategy = get_merge_strategy_for_branches(
        conn,
        repo_model,
        &pr_model.base_branch,
        &pr_model.head_branch,
    );

    let mut actions = vec![];

    if matches!(step, StepLabel::AwaitingMerge) {
        match merge_pull_request(
            config,
            &repo_model.owner,
            &repo_model.name,
            pr_model.get_number(),
            &commit_title,
            "",
            strategy,
        )
        .await
        {
            Err(e) => {
                actions.push(ResultAction::AddReaction(GhReactionType::MinusOne));
                actions.push(ResultAction::PostComment(format!(
                    "Could not merge this pull request: _{}_",
                    e
                )));
            }
            _ => {
                actions.push(ResultAction::AddReaction(GhReactionType::PlusOne));
                actions.push(ResultAction::PostComment(format!(
                    "Pull request successfully merged by {}! (strategy: '{}')",
                    comment_author,
                    strategy.to_string()
                )));
            }
        }
    } else {
        actions.push(ResultAction::AddReaction(GhReactionType::MinusOne));
        actions.push(ResultAction::PostComment(
            "Pull request is not ready to merge.".into(),
        ));
    }

    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_actions(actions)
        .build())
}

/// Handle `AdminSync` command.
pub async fn handle_admin_sync_command(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
) -> Result<CommandExecutionResult> {
    let (pr, _sha) = synchronize_pull_request(
        config,
        conn,
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
    )
    .await?;
    *pr_model = pr;

    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .build())
}

/// Handle `SkipQA` command.
pub fn handle_skip_qa_command(
    conn: &DbConn,
    pr_model: &mut PullRequestModel,
    status: bool,
) -> Result<CommandExecutionResult> {
    if status {
        pr_model.set_qa_status(QaStatus::Skipped);
    } else {
        pr_model.set_qa_status(QaStatus::Waiting);
    }

    pr_model.save(conn)?;

    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .build())
}

/// Handle `QaStatus` command.
pub async fn handle_qa_command(
    conn: &DbConn,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
    status: Option<bool>,
) -> Result<CommandExecutionResult> {
    let (status, status_text) = match status {
        Some(true) => (QaStatus::Pass, "marked as pass"),
        Some(false) => (QaStatus::Fail, "marked as fail"),
        None => (QaStatus::Waiting, "marked as waiting"),
    };

    pr_model.set_qa_status(status);
    pr_model.save(conn)?;

    let comment = format!("QA is {} by **{}**", status_text, comment_author);
    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

/// Handle `Ping` command.
pub async fn handle_ping_command(comment_author: &str) -> Result<CommandExecutionResult> {
    let comment = format!("**{}** pong!", comment_author);
    Ok(CommandExecutionResult::builder()
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

/// Handle `Gif` command.
pub async fn handle_gif_command(
    config: &Config,
    search_terms: &str,
) -> Result<CommandExecutionResult> {
    Ok(CommandExecutionResult::builder()
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(
            generate_random_gif_comment(config, search_terms).await?,
        ))
        .build())
}

/// Handle `AssignRequiredReviewers` command.
pub async fn handle_assign_required_reviewers_command(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    reviewers: Vec<String>,
) -> Result<CommandExecutionResult> {
    info!(
        pull_request_number = pr_model.get_number(),
        reviewers = ?reviewers,
        message = "Request required reviewers",
    );

    // Communicate to GitHub
    request_reviewers_for_pull_request(
        config,
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
        &reviewers,
    )
    .await?;

    for reviewer in &reviewers {
        ReviewModel::builder(repo_model, pr_model, reviewer)
            .required(true)
            .create_or_update(conn)?;
    }

    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .build())
}

/// Handle `UnassignRequiredReviewers` command.
pub async fn handle_unassign_required_reviewers_command(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    reviewers: Vec<String>,
) -> Result<CommandExecutionResult> {
    info!(
        pull_request_number = pr_model.get_number(),
        reviewers = ?reviewers,
        message = "Remove required reviewers",
    );

    remove_reviewers_for_pull_request(
        config,
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
        &reviewers,
    )
    .await?;

    for reviewer in &reviewers {
        ReviewModel::builder(repo_model, pr_model, reviewer)
            .required(false)
            .create_or_update(conn)?;
    }

    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .build())
}

/// Handle `Lock` command.
pub async fn handle_lock_command(
    conn: &DbConn,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
    status: bool,
    reason: Option<String>,
) -> Result<CommandExecutionResult> {
    let status_text = if status { "locked" } else { "unlocked" };

    pr_model.locked = status;
    pr_model.save(conn)?;

    let mut comment = format!("Pull request {} by **{}**", status_text, comment_author);
    if let Some(reason) = reason {
        comment = format!("{}\n**Reason**: {}", comment, reason);
    }

    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

/// Handle "Set needed reviewers" command.
pub async fn handle_set_needed_reviewers_command(
    conn: &DbConn,
    pr_model: &mut PullRequestModel,
    count: u32,
) -> Result<CommandExecutionResult> {
    pr_model.needed_reviewers_count = count as i32;
    pr_model.save(&conn)?;

    let comment = format!("Needed reviewers count set to **{}** for this PR.", count);
    Ok(CommandExecutionResult::builder()
        .with_status_update(true)
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

/// Handle `Help` command.
pub async fn handle_help_command(
    config: &Config,
    comment_author: &str,
) -> Result<CommandExecutionResult> {
    let comment = format!(
        "Hello **{}** ! I am a GitHub helper bot ! :robot:\n\
        You can ping me with a command in the format: `{} <command> (<arguments>)`\n\
        \n\
        Supported commands:\n\
        - `noqa+`: _Skip QA validation_\n\
        - `noqa-`: _Enable QA validation_\n\
        - `qa+`: _Mark QA as passed_\n\
        - `qa-`: _Mark QA as failed_\n\
        - `qa?`: _Mark QA as waiting_\n\
        - `automerge+`: _Enable auto-merge for this PR (once all checks pass)_\n\
        - `automerge-`: _Disable auto-merge for this PR_\n\
        - `lock+ <reason?>`: _Lock a pull-request (block merge)_\n\
        - `lock- <reason?>`: _Unlock a pull-request (unblock merge)_\n\
        - `req+ <reviewers>`: _Assign required reviewers (you can assign multiple reviewers)_\n\
        - `req- <reviewers>`: _Unassign required reviewers (you can unassign multiple reviewers)_\n\
        - `merge`: _Try merging the pull request_\n\
        - `ping`: _Ping me_\n\
        - `gif <search>`: _Post a random GIF with a tag_\n\
        - `help`: _Show this comment_\n",
        comment_author, config.bot_username
    );

    Ok(CommandExecutionResult::builder()
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

/// Handle `AdminHelp` command.
pub async fn handle_admin_help_command(
    config: &Config,
    comment_author: &str,
) -> Result<CommandExecutionResult> {
    let comment = format!(
        "Hello **{}** ! I am a GitHub helper bot ! :robot:\n\
        You can ping me with a command in the format: `{} <command> (<arguments>)`\n\
        \n\
        Supported admin commands:\n\
        - `admin-help`: _Show this comment_\n\
        - `admin-enable`: _Enable me on a pull request with manual interaction_\n\
        - `admin-set-needed-reviewers`: _Set needed reviewers count for this PR_\n\
        - `admin-sync`: _Update status comment if needed (maintenance-type command)_\n",
        comment_author, config.bot_username
    );

    Ok(CommandExecutionResult::builder()
        .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
        .with_action(ResultAction::PostComment(comment))
        .build())
}

#[cfg(test)]
mod tests {
    use github_scbot_database::{models::AccountModel, tests::using_test_db, Result};
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::LogicError;

    fn create_test_config() -> Config {
        let mut config = Config::from_env();
        config.bot_username = "test-bot".into();
        config.api_disable_client = true;
        config
    }

    #[actix_rt::test]
    async fn test_validate_user_rights_on_command() -> Result<()> {
        let config = create_test_config();

        using_test_db(&config.clone(), "test_logic_commands", |pool| async move {
            let conn = pool.get().unwrap();
            let creator = "me";
            let repo = RepositoryModel::builder(&config, "me", "test")
                .create_or_update(&conn)
                .unwrap();

            let pr = PullRequestModel::builder(&repo, 1, creator)
                .create_or_update(&conn)
                .unwrap();

            AccountModel::builder("non-admin")
                .admin(false)
                .create_or_update(&conn)
                .unwrap();

            AccountModel::builder("admin")
                .admin(true)
                .create_or_update(&conn)
                .unwrap();

            // PR creator should be valid
            assert_eq!(
                validate_user_rights_on_command(&conn, creator, &pr, &Command::Merge).unwrap(),
                true
            );
            // Non-admin should be invalid
            assert_eq!(
                validate_user_rights_on_command(&conn, "non-admin", &pr, &Command::Merge).unwrap(),
                false
            );
            // Admin should be valid
            assert_eq!(
                validate_user_rights_on_command(&conn, "admin", &pr, &Command::Merge).unwrap(),
                true
            );

            Ok::<_, LogicError>(())
        })
        .await
    }

    #[test]
    fn test_parse_command_string_from_comment_line() {
        let config = create_test_config();

        assert_eq!(
            parse_command_string_from_comment_line(
                &config,
                &format!("{} this-is-a-command", config.bot_username)
            ),
            Some(("this-is-a-command", vec![]))
        );

        assert_eq!(
            parse_command_string_from_comment_line(
                &config,
                &format!("{} lock+ Because I choosed to", config.bot_username)
            ),
            Some(("lock+", vec!["Because", "I", "choosed", "to"]))
        );

        assert_eq!(
            parse_command_string_from_comment_line(&config, "this-is-a-command"),
            None
        )
    }

    #[test]
    fn test_command_from_comment() {
        assert_eq!(
            Command::from_comment("noqa+", &Vec::new()),
            Some(Command::SkipQaStatus(true))
        );
        assert_eq!(
            Command::from_comment("noqa-", &Vec::new()),
            Some(Command::SkipQaStatus(false))
        );
        assert_eq!(
            Command::from_comment("qa+", &Vec::new()),
            Some(Command::QaStatus(Some(true)))
        );
        assert_eq!(
            Command::from_comment("qa-", &Vec::new()),
            Some(Command::QaStatus(Some(false)))
        );
        assert_eq!(
            Command::from_comment("qa?", &Vec::new()),
            Some(Command::QaStatus(None))
        );
        assert_eq!(
            Command::from_comment("automerge+", &Vec::new()),
            Some(Command::Automerge(true))
        );
        assert_eq!(
            Command::from_comment("automerge-", &Vec::new()),
            Some(Command::Automerge(false))
        );
        assert_eq!(
            Command::from_comment("this-is-a-command", &Vec::new()),
            None
        );
    }

    #[test]
    fn test_merge_command_results() {
        let results = vec![
            CommandExecutionResult::builder()
                .denied()
                .with_action(ResultAction::AddReaction(GhReactionType::MinusOne))
                .build(),
            CommandExecutionResult::builder()
                .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
                .with_action(ResultAction::PostComment("Comment 1".into()))
                .build(),
            CommandExecutionResult::builder().ignored().build(),
            CommandExecutionResult::builder()
                .with_status_update(true)
                .with_action(ResultAction::AddReaction(GhReactionType::Eyes))
                .with_action(ResultAction::PostComment("Comment 2".into()))
                .build(),
        ];

        let merged = merge_command_results(results);
        assert_eq!(
            merged,
            CommandExecutionResult {
                handling_status: CommandHandlingStatus::Handled,
                result_actions: vec![
                    ResultAction::AddReaction(GhReactionType::MinusOne),
                    ResultAction::AddReaction(GhReactionType::Eyes),
                    ResultAction::PostComment("Comment 1\n\n---\n\nComment 2".into())
                ],
                should_update_status: true
            }
        );
    }

    #[test]
    fn test_merge_command_results_ignored() {
        let results = vec![
            CommandExecutionResult::builder().ignored().build(),
            CommandExecutionResult::builder().ignored().build(),
        ];

        let merged = merge_command_results(results);
        assert_eq!(
            merged,
            CommandExecutionResult {
                handling_status: CommandHandlingStatus::Ignored,
                result_actions: vec![],
                should_update_status: false
            }
        );
    }
}
