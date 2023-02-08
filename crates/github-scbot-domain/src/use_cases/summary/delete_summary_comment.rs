use github_scbot_database::DbServiceAll;
use github_scbot_ghapi::adapter::ApiService;

use crate::Result;

use super::utils::sender::SummaryCommentSender;

pub struct DeleteSummaryCommentUseCase<'a> {
    pub api_service: &'a dyn ApiService,
    pub db_service: &'a mut dyn DbServiceAll,
    pub repo_owner: &'a str,
    pub repo_name: &'a str,
    pub pr_number: u64,
}

impl<'a> DeleteSummaryCommentUseCase<'a> {
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
