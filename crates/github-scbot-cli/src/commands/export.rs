use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use async_trait::async_trait;
use clap::Parser;
use github_scbot_database_interface::Exchanger;

use super::{Command, CommandContext};
use crate::Result;

/// Export all data
#[derive(Parser)]
pub(crate) struct ExportCommand {
    /// Output file, stdout if not precised
    #[clap(short, long)]
    output_file: Option<PathBuf>,
}

#[async_trait(?Send)]
impl Command for ExportCommand {
    async fn execute<W: Write>(self, ctx: CommandContext<W>) -> Result<()> {
        if let Some(file_path) = self.output_file {
            let file = File::create(file_path.clone())?;
            let mut writer = BufWriter::new(file);
            Exchanger::export_to_json(ctx.db_service.as_ref(), &mut writer)
                .await
                .map_err(Into::into)
        } else {
            let mut writer = std::io::stdout();
            Exchanger::export_to_json(ctx.db_service.as_ref(), &mut writer)
                .await
                .map_err(Into::into)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufWriter, Cursor};

    use github_scbot_config::Config;
    use github_scbot_database_interface::Exchanger;
    use github_scbot_database_tests::db_test_case;
    use github_scbot_domain_models::{
        Account, ExternalAccount, ExternalAccountRight, MergeRule, PullRequest, Repository,
        RequiredReviewer,
    };

    #[tokio::test]
    async fn test() {
        db_test_case("command_export", |db| async move {
            let config = Config::from_env();

            let repo = db
                .repositories_create(
                    Repository {
                        owner: "owner".into(),
                        name: "name".into(),
                        ..Default::default()
                    }
                    .with_config(&config),
                )
                .await?;
            let pr = db
                .pull_requests_create(
                    PullRequest {
                        number: 1,
                        ..Default::default()
                    }
                    .with_repository(&repo),
                )
                .await?;
            db.merge_rules_create(MergeRule {
                repository_id: repo.id,
                ..Default::default()
            })
            .await?;
            db.required_reviewers_create(RequiredReviewer {
                pull_request_id: pr.id,
                ..Default::default()
            })
            .await?;
            db.accounts_create(Account {
                username: "me".into(),
                is_admin: false,
            })
            .await?;
            db.external_accounts_create(ExternalAccount {
                username: "ext".into(),
                ..Default::default()
            })
            .await?;
            db.external_account_rights_create(ExternalAccountRight {
                repository_id: repo.id,
                username: "ext".into(),
            })
            .await?;

            let mut s = Vec::new();
            {
                let mut writer = BufWriter::new(&mut s);
                Exchanger::export_to_json(db.as_ref(), &mut writer).await?;
            }

            let cursor = Cursor::new(&s);
            Exchanger::import_from_json(db.as_ref(), cursor).await?;

            Ok(())
        })
        .await;
    }
}
