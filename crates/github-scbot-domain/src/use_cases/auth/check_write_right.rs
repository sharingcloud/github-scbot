use github_scbot_core::types::common::GhUserPermission;
use github_scbot_database::DbServiceAll;

use crate::Result;

use super::check_is_admin::CheckIsAdminUseCase;

pub struct CheckWriteRightUseCase<'a> {
    pub username: &'a str,
    pub user_permission: GhUserPermission,
    pub db_service: &'a mut dyn DbServiceAll,
}

impl<'a> CheckWriteRightUseCase<'a> {
    pub async fn run(&mut self) -> Result<bool> {
        let is_admin = CheckIsAdminUseCase {
            username: self.username,
            db_service: self.db_service,
        }
        .run()
        .await?;

        Ok(is_admin || self.user_permission.can_write())
    }
}
