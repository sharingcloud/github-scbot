//! Repository commands.

use std::convert::TryFrom;

use github_scbot_conf::Config;
use github_scbot_database::models::{IDatabaseAdapter, MergeRuleModel, RepositoryModel};
use github_scbot_libs::{dialoguer::Confirm, owo_colors::OwoColorize};
use github_scbot_types::pulls::GhMergeStrategy;

use super::errors::{CommandError, Result};

pub(crate) async fn add_repository(
    config: &Config,
    db_adapter: &dyn IDatabaseAdapter,
    repository_path: &str,
) -> Result<()> {
    let (owner, name) = RepositoryModel::extract_owner_and_name_from_path(repository_path)?;
    RepositoryModel::builder(config, owner, name)
        .create_or_update(db_adapter.repository())
        .await?;
    println!("Repository {} created.", repository_path);
    Ok(())
}

pub(crate) async fn set_pull_request_title_regex(
    db_adapter: &dyn IDatabaseAdapter,
    repository_path: &str,
    value: &str,
) -> Result<()> {
    let mut repo = RepositoryModel::get_from_path(db_adapter.repository(), repository_path).await?;
    println!("Accessing repository {}", repository_path);
    println!("Setting value '{}' as PR title validation regex", value);
    repo.pr_title_validation_regex = value.to_owned();
    db_adapter.repository().save(&mut repo).await?;

    Ok(())
}

pub(crate) async fn show_repository(
    db_adapter: &dyn IDatabaseAdapter,
    repository_path: &str,
) -> Result<()> {
    let repo = RepositoryModel::get_from_path(db_adapter.repository(), repository_path).await?;
    println!("Accessing repository {}", repository_path);
    println!("{:#?}", repo);

    Ok(())
}

pub(crate) async fn list_repositories(db_adapter: &dyn IDatabaseAdapter) -> Result<()> {
    let repos = db_adapter.repository().list().await?;
    if repos.is_empty() {
        println!("No repository known.");
    } else {
        for repo in repos {
            println!("- {}/{}", repo.owner, repo.name);
        }
    }

    Ok(())
}

pub(crate) async fn list_merge_rules(
    db_adapter: &dyn IDatabaseAdapter,
    repository_path: &str,
) -> Result<()> {
    let repo = RepositoryModel::get_from_path(db_adapter.repository(), repository_path).await?;
    let default_strategy = repo.get_default_merge_strategy();
    let rules = db_adapter
        .merge_rule()
        .list_from_repository_id(repo.id)
        .await?;

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

pub(crate) async fn set_merge_rule(
    db_adapter: &dyn IDatabaseAdapter,
    repository_path: &str,
    base_branch: &str,
    head_branch: &str,
    strategy: &str,
) -> Result<()> {
    let strategy_enum = GhMergeStrategy::try_from(strategy)?;
    let mut repo = RepositoryModel::get_from_path(db_adapter.repository(), repository_path).await?;

    if base_branch == "*" && head_branch == "*" {
        // Update default strategy
        repo.set_default_merge_strategy(strategy_enum);
        db_adapter.repository().save(&mut repo).await?;

        println!(
            "Default strategy updated to '{}' for repository '{}'",
            strategy, repository_path
        );
    } else {
        MergeRuleModel::builder(&repo, base_branch, head_branch)
            .strategy(strategy_enum)
            .create_or_update(db_adapter.merge_rule())
            .await?;
        println!("Merge rule created/updated with '{}' for repository '{}' and branches '{}' (base) <- '{}' (head)", strategy, repository_path, base_branch, head_branch);
    }

    Ok(())
}

pub(crate) async fn remove_merge_rule(
    db_adapter: &dyn IDatabaseAdapter,
    repository_path: &str,
    base_branch: &str,
    head_branch: &str,
) -> Result<()> {
    let repo = RepositoryModel::get_from_path(db_adapter.repository(), repository_path).await?;

    if base_branch == "*" && head_branch == "*" {
        return Err(CommandError::CannotRemoveDefaultStrategy);
    } else {
        // Try to get rule
        let rule = MergeRuleModel::get_from_branches(
            db_adapter.merge_rule(),
            &repo,
            base_branch,
            head_branch,
        )
        .await?;
        db_adapter.merge_rule().remove(rule).await?;
        println!(
            "Merge rule for repository '{}' and branches '{}' (base) <- '{}' (head) deleted.",
            repository_path, base_branch, head_branch
        );
    }

    Ok(())
}

pub(crate) async fn set_reviewers_count(
    db_adapter: &dyn IDatabaseAdapter,
    repository_path: &str,
    reviewers_count: u32,
) -> Result<()> {
    let mut repo = RepositoryModel::get_from_path(db_adapter.repository(), repository_path).await?;

    repo.default_needed_reviewers_count = reviewers_count as i32;
    println!(
        "Default reviewers count updated to {} for repository {}.",
        reviewers_count, repository_path
    );
    db_adapter.repository().save(&mut repo).await?;

    Ok(())
}

pub(crate) async fn purge_pull_requests(
    db_adapter: &dyn IDatabaseAdapter,
    repository_path: &str,
) -> Result<()> {
    let repo = RepositoryModel::get_from_path(db_adapter.repository(), repository_path).await?;

    let prs_to_purge = db_adapter
        .pull_request()
        .list_closed_pulls_from_repository(repo.id)
        .await?;
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
            db_adapter
                .pull_request()
                .remove_closed_pulls_from_repository(repo.id)
                .await?;
            println!("{} pull requests removed.", prs_to_purge.len());
        } else {
            println!("Cancelled.");
        }
    }

    Ok(())
}

pub(crate) async fn set_manual_interaction_mode(
    db_adapter: &dyn IDatabaseAdapter,
    repository_path: &str,
    manual_interaction: bool,
) -> Result<()> {
    let mut repo = RepositoryModel::get_from_path(db_adapter.repository(), repository_path).await?;
    repo.manual_interaction = manual_interaction;
    db_adapter.repository().save(&mut repo).await?;

    println!(
        "Manual interaction mode set to '{}' for repository {}.",
        manual_interaction, repository_path
    );

    Ok(())
}
