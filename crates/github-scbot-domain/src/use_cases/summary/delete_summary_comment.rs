use github_scbot_database_interface::DbService;
use github_scbot_ghapi_interface::ApiService;

use super::utils::sender::SummaryCommentSender;
use crate::Result;

pub struct DeleteSummaryCommentUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a mut dyn DbService,
    pub repo_owner: &'a str,
    pub repo_name: &'a str,
    pub pr_number: u64,
}

impl<'a> DeleteSummaryCommentUseCase<'a> {
    #[tracing::instrument(skip(self), fields(repo_owner, repo_name, pr_number))]
    pub async fn run(&mut self) -> Result<()> {
        SummaryCommentSender::delete(
            self.api_service,
            self.db_service,
            self.repo_owner,
            self.repo_name,
            self.pr_number,
        )
        .await
        .map(|_| ())
    }
}
