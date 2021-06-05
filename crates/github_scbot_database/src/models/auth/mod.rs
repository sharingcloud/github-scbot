//! Authentication models.

mod account;
mod external_account;
mod external_account_right;

pub use account::{AccountDbAdapter, AccountModel, DummyAccountDbAdapter, IAccountDbAdapter};
pub use external_account::{
    DummyExternalAccountDbAdapter, ExternalAccountDbAdapter, ExternalAccountModel,
    ExternalJwtClaims, IExternalAccountDbAdapter,
};
pub use external_account_right::{
    DummyExternalAccountRightDbAdapter, ExternalAccountRightDbAdapter, ExternalAccountRightModel,
    IExternalAccountRightDbAdapter,
};
