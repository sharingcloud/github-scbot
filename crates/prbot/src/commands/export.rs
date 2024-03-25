use std::{fs::File, io::BufWriter, path::PathBuf};

use async_trait::async_trait;
use clap::Parser;
use prbot_database_interface::Exchanger;

use super::{Command, CommandContext};
use crate::Result;

/// Export all data
#[derive(Parser)]
pub(crate) struct ExportCommand {
    /// Output file, stdout if not precised
    #[clap(short, long)]
    output_file: Option<PathBuf>,
}

#[async_trait]
impl Command for ExportCommand {
    async fn execute(self, ctx: CommandContext) -> Result<()> {
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

    use prbot_config::Config;
    use prbot_database_interface::{DbService, Exchanger};
    use prbot_database_memory::MemoryDb;
    use prbot_models::{
        Account, ExternalAccount, ExternalAccountRight, MergeRule, PullRequest, Repository,
        RequiredReviewer,
    };

    #[tokio::test]
    async fn test() {
        let db = MemoryDb::new();
        let config = Config::from_env_no_version();

        let repo = db
            .repositories_create(
                Repository {
                    owner: "owner".into(),
                    name: "name".into(),
                    ..Default::default()
                }
                .with_config(&config),
            )
            .await
            .unwrap();
        let pr = db
            .pull_requests_create(
                PullRequest {
                    number: 1,
                    ..Default::default()
                }
                .with_repository(&repo),
            )
            .await
            .unwrap();
        db.merge_rules_create(MergeRule {
            repository_id: repo.id,
            ..Default::default()
        })
        .await
        .unwrap();
        db.required_reviewers_create(RequiredReviewer {
            pull_request_id: pr.id,
            ..Default::default()
        })
        .await
        .unwrap();
        db.accounts_create(Account {
            username: "me".into(),
            is_admin: false,
        })
        .await
        .unwrap();
        db.external_accounts_create(ExternalAccount {
            username: "ext".into(),
            ..Default::default()
        })
        .await
        .unwrap();
        db.external_account_rights_create(ExternalAccountRight {
            repository_id: repo.id,
            username: "ext".into(),
        })
        .await
        .unwrap();

        let mut s = Vec::new();
        {
            let mut writer = BufWriter::new(&mut s);
            Exchanger::export_to_json(&db, &mut writer).await.unwrap();
        }

        let cursor = Cursor::new(&s);
        Exchanger::import_from_json(&db, cursor).await.unwrap();
    }
}
