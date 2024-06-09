use std::sync::Arc;
use crate::database::DatabaseConnection;
use crate::rest::account_rest::AccountRest;
use crate::service::account_service::AccountService;


// #[derive(Clone)]  // TODO: use derive
pub struct Dependencies <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
> {
    pub database_connection: Arc<DatabaseConnection>,
    pub account_service: Arc<AccountS>,
    pub account_rest: Arc<AccountRest<AccountS>>,
}


impl <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
> Clone for Dependencies<AccountS> {
    fn clone(&self) -> Self {
        Dependencies::<AccountS> {
            database_connection: Arc::clone(&self.database_connection),
            account_service: Arc::clone(&self.account_service),
            account_rest: Arc::clone(&self.account_rest),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.database_connection = Arc::clone(&source.database_connection);
        self.account_service = Arc::clone(&source.account_service);
        self.account_rest = Arc::clone(&source.account_rest);
    }
}
