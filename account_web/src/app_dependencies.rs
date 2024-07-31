use std::sync::Arc;
use crate::{
    auth::ClientAuthUserProvider,
    service::account_service::AccountService,
};
//--------------------------------------------------------------------------------------------------



pub struct DependenciesState <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
> {
    // pub database_connection: Arc<sqlx_postgres::PgPool>,
    pub account_service: Arc<AccountS>,
    //pub account_rest: Arc<AccountRest<AccountS>>,
    //pub user_perm_provider: Arc<SqlUserProvider>,
    pub user_perm_provider: Arc<ClientAuthUserProvider>,
}

pub struct Dependencies <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
> {
    pub state: Arc<DependenciesState<AccountS>>,
}

impl <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
> Clone for Dependencies<AccountS> {
    fn clone(&self) -> Self {
        Dependencies::<AccountS> { state: Arc::clone(&self.state) }
    }
    fn clone_from(&mut self, source: &Self) {
        self.state = Arc::clone(&source.state)
    }
}
