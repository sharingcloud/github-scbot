//! Repository commands.

use std::convert::TryFrom;

use anyhow::Result;
use github_scbot_database::{
    establish_single_connection,
    models::{MergeRuleCreation, MergeRuleModel, RepositoryModel},
};
use github_scbot_types::pulls::GHMergeStrategy;

/// Set the pull request title validation regex for a repository.
///
/// # Arguments
///
/// * `repository_path` - Repository path (<owner>/<name>)
/// * `value` - Regex value
pub fn set_pull_request_title_regex(repository_path: &str, value: &str) -> Result<()> {
    let conn = establish_single_connection()?;

    if let Some(mut repo) = RepositoryModel::get_from_path(&conn, &repository_path)? {
        println!("Accessing repository {}", repository_path);
        println!("Setting value '{}' as PR title validation regex", value);
        repo.pr_title_validation_regex = value.to_owned();
        repo.save(&conn)?;
    } else {
        eprintln!("Unknown repository {}.", repository_path);
    }

    Ok(())
}

/// Show repository data stored in database.
///
/// # Arguments
///
/// * `repository_path` - Repository path (<owner>/<name>)
pub fn show_repository(repository_path: &str) -> Result<()> {
    let conn = establish_single_connection()?;

    if let Some(repo) = RepositoryModel::get_from_path(&conn, &repository_path)? {
        println!("Accessing repository {}", repository_path);
        println!("{:#?}", repo);
    } else {
        eprintln!("Unknown repository {}.", repository_path);
    }

    Ok(())
}

/// List known repositories from database.
pub fn list_repositories() -> Result<()> {
    let conn = establish_single_connection()?;

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

/// List merge rules for repository.
///
/// # Arguments
///
/// * `repository_path` - Repository path
pub fn list_merge_rules(repository_path: &str) -> Result<()> {
    let conn = establish_single_connection()?;

    if let Some(repo) = RepositoryModel::get_from_path(&conn, &repository_path)? {
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
    } else {
        eprintln!("Unknown repository {}.", repository_path);
    }

    Ok(())
}

/// Set merge rule for repository.
///
/// # Arguments
///
/// * `repository_path` - Repository path
/// * `base_branch` - Base branch
/// * `head_branch` - Head branch
/// * `strategy` - Merge strategy
pub fn set_merge_rule(
    repository_path: &str,
    base_branch: &str,
    head_branch: &str,
    strategy: &str,
) -> Result<()> {
    let conn = establish_single_connection()?;
    let strategy_enum = GHMergeStrategy::try_from(strategy)?;

    if let Some(mut repo) = RepositoryModel::get_from_path(&conn, &repository_path)? {
        if base_branch == "*" && head_branch == "*" {
            // Update default strategy
            repo.set_default_merge_strategy(strategy_enum);
            repo.save(&conn)?;

            println!(
                "Default strategy updated to '{}' for repository '{}'",
                strategy, repository_path
            );
        } else {
            // Try to get rule
            if let Some(mut rule) =
                MergeRuleModel::get_from_branches(&conn, repo.id, base_branch, head_branch)
            {
                rule.set_strategy(strategy_enum);
                rule.save(&conn)?;
                println!("Merge rule updated to '{}' for repository '{}' and branches '{}' (base) <- '{}' (head)", strategy, repository_path, base_branch, head_branch);
            } else {
                MergeRuleModel::create(
                    &conn,
                    MergeRuleCreation {
                        repository_id: repo.id,
                        base_branch: base_branch.into(),
                        head_branch: head_branch.into(),
                        strategy: strategy.into(),
                    },
                )?;
                println!("Merge rule created with '{}' for repository '{}' and branches '{}' (base) <- '{}' (head)", strategy, repository_path, base_branch, head_branch);
            }
        }
    } else {
        eprintln!("Unknown repository {}.", repository_path);
    }

    Ok(())
}

/// Remove merge rule for repository.
///
/// # Arguments
///
/// * `repository_path` - Repository path
/// * `base_branch` - Base branch
/// * `head_branch` - Head branch
pub fn remove_merge_rule(
    repository_path: &str,
    base_branch: &str,
    head_branch: &str,
) -> Result<()> {
    let conn = establish_single_connection()?;

    if let Some(repo) = RepositoryModel::get_from_path(&conn, &repository_path)? {
        if base_branch == "*" && head_branch == "*" {
            eprintln!("You cannot remove default strategy.");
        } else {
            // Try to get rule
            if let Some(rule) =
                MergeRuleModel::get_from_branches(&conn, repo.id, base_branch, head_branch)
            {
                rule.remove(&conn)?;
                println!("Merge rule for repository '{}' and branches '{}' (base) <- '{}' (head) deleted.", repository_path, base_branch, head_branch);
            } else {
                println!("Merge rule not found for repository '{}' and branches '{}' (base) <- '{}' (head).", repository_path, base_branch, head_branch);
            }
        }
    } else {
        eprintln!("Unknown repository {}.", repository_path);
    }

    Ok(())
}

/// Set reviewers count for repository.
///
/// # Arguments
///
/// * `repository_path` - Repository path
/// * `reviewers_count` - Reviewers count
pub fn set_reviewers_count(repository_path: &str, reviewers_count: u32) -> Result<()> {
    let conn = establish_single_connection()?;

    if let Some(mut repo) = RepositoryModel::get_from_path(&conn, &repository_path)? {
        repo.default_needed_reviewers_count = reviewers_count as i32;
        println!(
            "Default reviewers count updated to {} for repository {}.",
            reviewers_count, repository_path
        );
        repo.save(&conn)?;
    } else {
        eprintln!("Unknown repository {}.", repository_path);
    }

    Ok(())
}
