use std::{
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use crate::Result;
use async_trait::async_trait;
use clap::Parser;
use github_scbot_database::Exchanger;

use super::{Command, CommandContext};

/// Export all data
#[derive(Parser)]
pub(crate) struct ExportCommand {
    /// Output file, stdout if not precised
    #[clap(short, long)]
    output_file: Option<PathBuf>,
}

#[async_trait(?Send)]
impl Command for ExportCommand {
    async fn execute<W: Write>(self, mut ctx: CommandContext<W>) -> Result<()> {
        if let Some(file_path) = self.output_file {
            let file = File::create(file_path.clone())?;
            let mut writer = BufWriter::new(file);
            Exchanger::export_to_json(&mut *ctx.db_adapter, &mut writer)
                .await
                .map_err(Into::into)
        } else {
            let mut writer = std::io::stdout();
            Exchanger::export_to_json(&mut *ctx.db_adapter, &mut writer)
                .await
                .map_err(Into::into)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufWriter, Cursor};

    use github_scbot_core::config::Config;
    use github_scbot_database::{
        use_temporary_db, Account, DbService, DbServiceImplPool, Exchanger, ExternalAccount,
        ExternalAccountRight, MergeRule, PullRequest, Repository, RequiredReviewer,
    };

    #[actix_rt::test]
    async fn test() {
        let config = Config::from_env();
        use_temporary_db(config, "test_command_export", |config, pool| async move {
            let mut db_adapter = DbServiceImplPool::new(pool);
            let repo = db_adapter
                .repositories()
                .create(Repository::builder().with_config(&config).build()?)
                .await?;
            let pr = db_adapter
                .pull_requests()
                .create(PullRequest::builder().with_repository(&repo).build()?)
                .await?;
            db_adapter
                .merge_rules()
                .create(MergeRule::builder().with_repository(&repo).build()?)
                .await?;
            db_adapter
                .required_reviewers()
                .create(RequiredReviewer::builder().with_pull_request(&pr).build()?)
                .await?;
            db_adapter
                .accounts()
                .create(Account::builder().build()?)
                .await?;
            db_adapter
                .external_accounts()
                .create(ExternalAccount::builder().build()?)
                .await?;
            db_adapter
                .external_account_rights()
                .create(
                    ExternalAccountRight::builder()
                        .with_repository(&repo)
                        .build()?,
                )
                .await?;

            let mut s = Vec::new();
            {
                let mut writer = BufWriter::new(&mut s);
                Exchanger::export_to_json(&mut db_adapter, &mut writer).await?;
            }

            let cursor = Cursor::new(&s);
            Exchanger::import_from_json(&mut db_adapter, cursor).await?;

            Ok(())
        })
        .await;
    }
}
