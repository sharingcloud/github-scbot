//! Commands module.

use github_scbot_api::{
    comments::{add_reaction_to_comment, post_comment},
    pulls::{get_pull_request_sha, merge_pull_request},
    reviews::{remove_reviewers_for_pull_request, request_reviewers_for_pull_request},
};
use github_scbot_conf::Config;
use github_scbot_database::{
    models::{PullRequestModel, RepositoryModel, ReviewModel},
    DbConn,
};
use github_scbot_types::{issues::GHReactionType, labels::StepLabel, status::QAStatus};
use tracing::info;

use super::{errors::Result, status::update_pull_request_status};
use crate::{
    auth::{has_right_on_pull_request, list_known_admin_usernames},
    gif::post_random_gif_comment,
    pulls::{determine_automatic_step, get_merge_strategy_for_branches, synchronize_pull_request},
};

/// Command handling status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandHandlingStatus {
    /// Command handled.
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
    SkipQAStatus(bool),
    /// Enable/Disable QA status.
    QAStatus(Option<bool>),
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
    /// Synchronize status.
    Synchronize,
}

impl Command {
    /// Create a command from a comment and arguments.
    ///
    /// # Arguments
    ///
    /// * `comment` - Comment
    /// * `args` - Arguments
    pub fn from_comment(comment: &str, args: &[&str]) -> Option<Self> {
        Some(match comment {
            "noqa+" => Self::SkipQAStatus(true),
            "noqa-" => Self::SkipQAStatus(false),
            "qa+" => Self::QAStatus(Some(true)),
            "qa-" => Self::QAStatus(Some(false)),
            "qa?" => Self::QAStatus(None),
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
            "sync" => Self::Synchronize,
            _ => return None,
        })
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
}

/// Parse commands from comment body.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `comment_id` - Comment ID
/// * `comment_author` - Comment author
/// * `comment_body` - Comment body
pub async fn parse_commands(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_id: u64,
    comment_author: &str,
    comment_body: &str,
) -> Result<CommandHandlingStatus> {
    let mut command_handling = CommandHandlingStatus::Ignored;

    for line in comment_body.lines() {
        let line_handling = parse_single_command(
            config,
            conn,
            repo_model,
            pr_model,
            comment_id,
            comment_author,
            line,
        )
        .await?;

        command_handling = match line_handling {
            CommandHandlingStatus::Ignored => command_handling,
            status => status,
        };
    }

    Ok(command_handling)
}

/// Parse command from a single comment line.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `comment_id` - Comment ID
/// * `comment_author` - Comment author
/// * `line` - Comment line
pub async fn parse_single_command(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_id: u64,
    comment_author: &str,
    line: &str,
) -> Result<CommandHandlingStatus> {
    let mut command_handled = CommandHandlingStatus::Ignored;

    if let Some((command_line, args)) = parse_command_string_from_comment_line(config, line) {
        let action = Command::from_comment(command_line, &args);
        if let Some(action) = action {
            info!(
                "Interpreting action {:?} from author {} on repository {}, PR #{}",
                action,
                comment_author,
                repo_model.get_path(),
                pr_model.get_number()
            );

            if !validate_user_rights_on_command(conn, comment_author, pr_model, &action)? {
                command_handled = CommandHandlingStatus::Denied;
            } else {
                command_handled = CommandHandlingStatus::Handled;

                let status_updated = match action {
                    Command::Automerge(s) => {
                        handle_auto_merge_command(
                            config,
                            conn,
                            repo_model,
                            pr_model,
                            comment_author,
                            s,
                        )
                        .await?
                    }
                    Command::SkipQAStatus(s) => handle_skip_qa_command(conn, pr_model, s)?,
                    Command::QAStatus(s) => {
                        handle_qa_command(config, conn, repo_model, pr_model, comment_author, s)
                            .await?
                    }
                    Command::Lock(s, reason) => {
                        handle_lock_command(
                            config,
                            conn,
                            repo_model,
                            pr_model,
                            comment_author,
                            s,
                            reason,
                        )
                        .await?
                    }
                    Command::Ping => {
                        handle_ping_command(config, repo_model, pr_model, comment_author).await?
                    }
                    Command::Merge => {
                        handle_merge_command(
                            config,
                            conn,
                            repo_model,
                            pr_model,
                            comment_id,
                            comment_author,
                        )
                        .await?
                    }
                    Command::Synchronize => {
                        handle_sync_command(config, conn, repo_model, pr_model).await?
                    }
                    Command::AssignRequiredReviewers(reviewers) => {
                        handle_assign_required_reviewers_command(
                            config, conn, repo_model, pr_model, reviewers,
                        )
                        .await?
                    }
                    Command::UnassignRequiredReviewers(reviewers) => {
                        handle_unassign_required_reviewers_command(
                            config, conn, repo_model, pr_model, reviewers,
                        )
                        .await?
                    }
                    Command::Gif(terms) => {
                        handle_gif_command(config, repo_model, pr_model, &terms).await?
                    }
                    Command::Help => {
                        handle_help_command(config, repo_model, pr_model, comment_author).await?
                    }
                };

                if status_updated {
                    let sha = get_pull_request_sha(
                        config,
                        &repo_model.owner,
                        &repo_model.name,
                        pr_model.get_number(),
                    )
                    .await?;
                    update_pull_request_status(config, conn, repo_model, pr_model, &sha).await?;
                }
            }
        }
    }

    Ok(command_handled)
}

/// Validate user rights on command.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `username` - Target username
/// * `pr_model` - Pull request
/// * `command` - Command
pub fn validate_user_rights_on_command(
    conn: &DbConn,
    username: &str,
    pr_model: &PullRequestModel,
    command: &Command,
) -> Result<bool> {
    match command {
        Command::Ping | Command::Help | Command::Gif(_) => Ok(true),
        _ => {
            let known_admins = list_known_admin_usernames(conn)?;
            Ok(has_right_on_pull_request(username, pr_model, &known_admins))
        }
    }
}

/// Parse command string from comment line.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `comment` - Comment
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
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `comment_author` - Comment author
/// * `status` - Automerge status
pub async fn handle_auto_merge_command(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
    status: bool,
) -> Result<bool> {
    pr_model.automerge = status;
    pr_model.save(conn)?;

    let status_text = if status { "enabled" } else { "disabled" };
    let comment = format!("Automerge {} by **{}**", status_text, comment_author);
    post_comment(
        config,
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
        &comment,
    )
    .await?;

    Ok(true)
}

/// Handle `Merge` command.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `comment_id` - Comment ID
/// * `comment_author` - Comment author
pub async fn handle_merge_command(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_id: u64,
    comment_author: &str,
) -> Result<bool> {
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
                add_reaction_to_comment(
                    config,
                    &repo_model.owner,
                    &repo_model.name,
                    comment_id,
                    GHReactionType::MinusOne,
                )
                .await?;
                post_comment(
                    config,
                    &repo_model.owner,
                    &repo_model.name,
                    pr_model.get_number(),
                    &format!("Could not merge this pull request: _{}_", e),
                )
                .await?;
            }
            _ => {
                add_reaction_to_comment(
                    config,
                    &repo_model.owner,
                    &repo_model.name,
                    comment_id,
                    GHReactionType::PlusOne,
                )
                .await?;
                post_comment(
                    config,
                    &repo_model.owner,
                    &repo_model.name,
                    pr_model.get_number(),
                    &format!(
                        "Pull request successfully merged by {}! (strategy: '{}')",
                        comment_author,
                        strategy.to_string()
                    ),
                )
                .await?;
            }
        }
    } else {
        add_reaction_to_comment(
            config,
            &repo_model.owner,
            &repo_model.name,
            comment_id,
            GHReactionType::MinusOne,
        )
        .await?;
        post_comment(
            config,
            &repo_model.owner,
            &repo_model.name,
            pr_model.get_number(),
            "Pull request is not ready to merge.",
        )
        .await?;
    }

    Ok(true)
}

/// Handle `Sync` command.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
pub async fn handle_sync_command(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
) -> Result<bool> {
    let (pr, _sha) = synchronize_pull_request(
        config,
        conn,
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
    )
    .await?;
    *pr_model = pr;
    Ok(true)
}

/// Handle `SkipQA` command.
///
/// # Arguments
///
/// * `conn` - Database connection
/// * `pr_model` - Pull request model
/// * `status` - Skip QA status
pub fn handle_skip_qa_command(
    conn: &DbConn,
    pr_model: &mut PullRequestModel,
    status: bool,
) -> Result<bool> {
    if status {
        pr_model.set_qa_status(QAStatus::Skipped);
    } else {
        pr_model.set_qa_status(QAStatus::Waiting);
    }

    pr_model.save(conn)?;

    Ok(true)
}

/// Handle `QAStatus` command.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `comment_author` - Comment author
/// * `status` - QA status
pub async fn handle_qa_command(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
    status: Option<bool>,
) -> Result<bool> {
    let (status, status_text) = match status {
        Some(true) => (QAStatus::Pass, "marked as pass"),
        Some(false) => (QAStatus::Fail, "marked as fail"),
        None => (QAStatus::Waiting, "marked as waiting"),
    };

    pr_model.set_qa_status(status);
    pr_model.save(conn)?;

    let comment = format!("QA is {} by **{}**", status_text, comment_author);
    post_comment(
        config,
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
        &comment,
    )
    .await?;

    Ok(true)
}

/// Handle `Ping` command.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `comment_author` - Comment author
pub async fn handle_ping_command(
    config: &Config,
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    comment_author: &str,
) -> Result<bool> {
    post_comment(
        config,
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
        &format!("**{}** pong!", comment_author),
    )
    .await?;

    Ok(true)
}

/// Handle `Gif` command.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `search_terms` - Search terms
pub async fn handle_gif_command(
    config: &Config,
    repo_model: &RepositoryModel,
    pr_model: &PullRequestModel,
    search_terms: &str,
) -> Result<bool> {
    post_random_gif_comment(config, repo_model, pr_model, search_terms).await?;

    Ok(false)
}

/// Handle `AssignRequiredReviewers` command.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `reviewers` - Reviewers
pub async fn handle_assign_required_reviewers_command(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    reviewers: Vec<String>,
) -> Result<bool> {
    info!(
        "Request required reviewers for PR #{}: {:#?}",
        pr_model.get_number(),
        reviewers
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

    Ok(true)
}

/// Handle `UnassignRequiredReviewers` command.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `reviewers` - Reviewers
pub async fn handle_unassign_required_reviewers_command(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    reviewers: Vec<String>,
) -> Result<bool> {
    info!(
        "Remove required reviewers for PR #{}: {:#?}",
        pr_model.get_number(),
        reviewers
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

    Ok(true)
}

/// Handle `Lock` command.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `conn` - Database connection
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `comment_author` - Comment author
/// * `status` - Lock status
/// * `reason` - Optional lock motivation
pub async fn handle_lock_command(
    config: &Config,
    conn: &DbConn,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
    status: bool,
    reason: Option<String>,
) -> Result<bool> {
    let status_text = if status { "locked" } else { "unlocked" };

    pr_model.locked = status;
    pr_model.save(conn)?;

    let mut comment = format!("Pull request {} by **{}**", status_text, comment_author);
    if let Some(reason) = reason {
        comment = format!("{}\n**Reason**: {}", comment, reason);
    }

    post_comment(
        config,
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
        &comment,
    )
    .await?;

    Ok(true)
}

/// Handle `Help` command.
///
/// # Arguments
///
/// * `config` - Bot configuration
/// * `repo_model` - Repository model
/// * `pr_model` - Pull request model
/// * `comment_author` - Comment author
pub async fn handle_help_command(
    config: &Config,
    repo_model: &RepositoryModel,
    pr_model: &mut PullRequestModel,
    comment_author: &str,
) -> Result<bool> {
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
        - `help`: _Show this comment_\n\
        - `sync`: _Update status comment if needed (maintenance-type command)_\n",
        comment_author, config.bot_username
    );

    post_comment(
        config,
        &repo_model.owner,
        &repo_model.name,
        pr_model.get_number(),
        &comment,
    )
    .await?;

    Ok(false)
}

#[cfg(test)]
mod tests {
    use github_scbot_database::models::AccountModel;
    use github_scbot_database::tests::using_test_db;
    use github_scbot_database::Result;

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
            Some(Command::SkipQAStatus(true))
        );
        assert_eq!(
            Command::from_comment("noqa-", &Vec::new()),
            Some(Command::SkipQAStatus(false))
        );
        assert_eq!(
            Command::from_comment("qa+", &Vec::new()),
            Some(Command::QAStatus(Some(true)))
        );
        assert_eq!(
            Command::from_comment("qa-", &Vec::new()),
            Some(Command::QAStatus(Some(false)))
        );
        assert_eq!(
            Command::from_comment("qa?", &Vec::new()),
            Some(Command::QAStatus(None))
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
}
