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
            Exchanger::export_to_json(ctx.db_adapter.as_mut(), &mut writer)
                .await
                .map_err(Into::into)
        } else {
            let mut writer = std::io::stdout();
            Exchanger::export_to_json(ctx.db_adapter.as_mut(), &mut writer)
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
        db_test_case, Account, Exchanger, ExternalAccount, ExternalAccountRight, MergeRule,
        PullRequest, Repository, RequiredReviewer,
    };

    #[actix_rt::test]
    async fn test() {
        db_test_case("command_export", |mut db| async move {
            let config = Config::from_env();

            let repo = db
                .repositories_create(Repository::builder().with_config(&config).build()?)
                .await?;
            let pr = db
                .pull_requests_create(PullRequest::builder().with_repository(&repo).build()?)
                .await?;
            db.merge_rules_create(MergeRule::builder().with_repository(&repo).build()?)
                .await?;
            db.required_reviewers_create(
                RequiredReviewer::builder().with_pull_request(&pr).build()?,
            )
            .await?;
            db.accounts_create(Account::builder().build()?).await?;
            db.external_accounts_create(ExternalAccount::builder().build()?)
                .await?;
            db.external_account_rights_create(
                ExternalAccountRight::builder()
                    .with_repository(&repo)
                    .build()?,
            )
            .await?;

            let mut s = Vec::new();
            {
                let mut writer = BufWriter::new(&mut s);
                Exchanger::export_to_json(db.as_mut(), &mut writer).await?;
            }

            let cursor = Cursor::new(&s);
            println!("imp");
            Exchanger::import_from_json(db.as_mut(), cursor).await?;

            Ok(())
        })
        .await;
    }
}
