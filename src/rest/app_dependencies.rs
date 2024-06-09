use std::sync::Arc;
use crate::rest::account_rest::AccountRest;
use crate::service::account_service::AccountService;

pub struct Dependencies <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
> {
    pub account_service: Arc<AccountS>,
    pub account_rest: Arc<AccountRest<AccountS>>,
}



