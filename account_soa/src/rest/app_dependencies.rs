use std::sync::Arc;
use mvv_auth::PlainPasswordComparator;
use crate::rest::account_rest::AccountRest;
use crate::rest::auth::user_perm_provider::SqlUserProvider;
use crate::service::account_service::AccountService;
//--------------------------------------------------------------------------------------------------



pub struct DependenciesState <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
> {
    pub database_connection: Arc<sqlx_postgres::PgPool>,
    pub account_service: Arc<AccountS>,
    pub account_rest: Arc<AccountRest<AccountS>>,
    pub psw_comparator: Arc<PlainPasswordComparator>,
    pub user_perm_provider: Arc<SqlUserProvider>,
}

pub struct Dependencies <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
> {
    pub state: Arc<DependenciesState<AccountS>>,
}


//noinspection DuplicatedCode
impl <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
> Clone for Dependencies<AccountS> {
    fn clone(&self) -> Self {
        Dependencies::<AccountS> {
            state: Arc::clone(&self.state),
            /*
            database_connection: Arc::clone(&self.database_connection),
            account_service: Arc::clone(&self.account_service),
            account_rest: Arc::clone(&self.account_rest),
            user_perm_provider: Arc::clone(&self.user_perm_provider),
            */
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.state = Arc::clone(&source.state)
        /*
        self.database_connection = Arc::clone(&source.database_connection);
        self.account_service = Arc::clone(&source.account_service);
        self.account_rest = Arc::clone(&source.account_rest);
        */
    }
}
