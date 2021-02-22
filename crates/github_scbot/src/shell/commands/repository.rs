//! Repository commands.

use std::convert::TryFrom;

use dialoguer::Confirm;
use github_scbot_conf::Config;
use github_scbot_database::{
    establish_single_connection,
    models::{MergeRuleModel, PullRequestModel, RepositoryModel},
};
use github_scbot_types::pulls::GHMergeStrategy;
use owo_colors::OwoColorize;

use super::errors::{CommandError, Result};

pub(crate) fn set_pull_request_title_regex(
    config: &Config,
    repository_path: &str,
    value: &str,
) -> Result<()> {
    let conn = establish_single_connection(config)?;
    let mut repo = RepositoryModel::get_from_path(&conn, &repository_path)?;

    println!("Accessing repository {}", repository_path);
    println!("Setting value '{}' as PR title validation regex", value);
    repo.pr_title_validation_regex = value.to_owned();
    repo.save(&conn)?;

    Ok(())
}

pub(crate) fn show_repository(config: &Config, repository_path: &str) -> Result<()> {
    let conn = establish_single_connection(config)?;
    let repo = RepositoryModel::get_from_path(&conn, &repository_path)?;

    println!("Accessing repository {}", repository_path);
    println!("{:#?}", repo);

    Ok(())
}

pub(crate) fn list_repositories(config: &Config) -> Result<()> {
    let conn = establish_single_connection(config)?;

    let repos = RepositoryModel::list(&conn)?;
    if repos.is_empty() {
        println!("No repository known.");
    } else {
        for repo in repos {
            println!("- {}/{}", repo.owner, repo.name);
        }
    }

    Ok(())
}

pub(crate) fn list_merge_rules(config: &Config, repository_path: &str) -> Result<()> {
    let conn = establish_single_connection(config)?;
    let repo = RepositoryModel::get_from_path(&conn, &repository_path)?;
    let default_strategy = repo.get_default_merge_strategy();
    let rules = MergeRuleModel::list_from_repository_id(&conn, repo.id)?;

    println!("Merge rules for repository {}:", repository_path);
    println!("- Default: '{}'", default_strategy.to_string());
    for rule in rules {
        println!(
            "- '{}' (base) <- '{}' (head): '{}'",
            rule.base_branch,
            rule.head_branch,
            rule.get_strategy().to_string()
        );
    }

    Ok(())
}

pub(crate) fn set_merge_rule(
    config: &Config,
    repository_path: &str,
    base_branch: &str,
    head_branch: &str,
    strategy: &str,
) -> Result<()> {
    let conn = establish_single_connection(config)?;
    let strategy_enum = GHMergeStrategy::try_from(strategy)?;
    let mut repo = RepositoryModel::get_from_path(&conn, &repository_path)?;

    if base_branch == "*" && head_branch == "*" {
        // Update default strategy
        repo.set_default_merge_strategy(strategy_enum);
        repo.save(&conn)?;

        println!(
            "Default strategy updated to '{}' for repository '{}'",
            strategy, repository_path
        );
    } else {
        MergeRuleModel::builder(&repo, base_branch, head_branch)
            .strategy(strategy_enum)
            .create_or_update(&conn)?;
        println!("Merge rule created/updated with '{}' for repository '{}' and branches '{}' (base) <- '{}' (head)", strategy, repository_path, base_branch, head_branch);
    }

    Ok(())
}

pub(crate) fn remove_merge_rule(
    config: &Config,
    repository_path: &str,
    base_branch: &str,
    head_branch: &str,
) -> Result<()> {
    let conn = establish_single_connection(config)?;
    let repo = RepositoryModel::get_from_path(&conn, &repository_path)?;

    if base_branch == "*" && head_branch == "*" {
        return Err(CommandError::CannotRemoveDefaultStrategy);
    } else {
        // Try to get rule
        let rule = MergeRuleModel::get_from_branches(&conn, &repo, base_branch, head_branch)?;
        rule.remove(&conn)?;
        println!(
            "Merge rule for repository '{}' and branches '{}' (base) <- '{}' (head) deleted.",
            repository_path, base_branch, head_branch
        );
    }

    Ok(())
}

pub(crate) fn set_reviewers_count(
    config: &Config,
    repository_path: &str,
    reviewers_count: u32,
) -> Result<()> {
    let conn = establish_single_connection(config)?;
    let mut repo = RepositoryModel::get_from_path(&conn, &repository_path)?;

    repo.default_needed_reviewers_count = reviewers_count as i32;
    println!(
        "Default reviewers count updated to {} for repository {}.",
        reviewers_count, repository_path
    );
    repo.save(&conn)?;

    Ok(())
}

pub(crate) fn purge_pull_requests(config: &Config, repository_path: &str) -> Result<()> {
    let conn = establish_single_connection(config)?;
    let repo = RepositoryModel::get_from_path(&conn, &repository_path)?;

    let prs_to_purge = PullRequestModel::list_closed_pulls(&conn, repo.id)?;
    if prs_to_purge.is_empty() {
        println!(
            "No closed pull request to remove for repository '{}'",
            repository_path
        );
    } else {
        println!(
            "You will remove:\n{}",
            prs_to_purge
                .iter()
                .map(|p| format!("- #{} - {}", p.get_number(), p.name))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let prompt = "Do you want to continue?".yellow();
        if Confirm::new().with_prompt(prompt.to_string()).interact()? {
            PullRequestModel::remove_closed_pulls(&conn, repo.id)?;
            println!("{} pull requests removed.", prs_to_purge.len());
        } else {
            println!("Cancelled.");
        }
    }

    Ok(())
}
