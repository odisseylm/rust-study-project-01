use std::sync::Arc;
use mvv_auth::{PasswordComparator};
use crate::{
    auth::AuthUserProvider,
    service::account_service::AccountService,
};
//--------------------------------------------------------------------------------------------------



pub struct DependenciesState <
    AccountS: AccountService + Send + Sync + 'static,
    // AccountR: AccountRest<AccountS> + Send + Sync,
> {
    pub database_connection: Arc<sqlx_postgres::PgPool>,
    pub account_service: Arc<AccountS>,
    //pub account_rest: Arc<AccountRest<AccountS>>,
    pub psw_comp: Arc<dyn PasswordComparator + Send + Sync>,
    pub user_perm_provider: Arc<AuthUserProvider>,
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
        Dependencies::<AccountS> { state: Arc::clone(&self.state) }
    }
    fn clone_from(&mut self, source: &Self) {
        self.state = Arc::clone(&source.state)
    }
}
