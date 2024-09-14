use std::sync::Arc;
use mvv_auth::{PasswordComparator};
use crate::{
    auth::AuthUserProvider,
};
use crate::service::{
    account_service::AccountServiceImpl,
    client_search_service::ClientSearchServiceImpl,
};
//--------------------------------------------------------------------------------------------------



pub struct DependenciesState {
    pub database_connection: Arc<sqlx_postgres::PgPool>,
    pub account_service: Arc<AccountServiceImpl>,
    pub client_search_service: Arc<ClientSearchServiceImpl>,
    pub psw_comp: Arc<dyn PasswordComparator + Send + Sync>,
    pub user_perm_provider: Arc<AuthUserProvider>,
}

pub struct Dependencies {
    pub state: Arc<DependenciesState>,
}

//noinspection DuplicatedCode
impl Clone for Dependencies {
    fn clone(&self) -> Self {
        Dependencies { state: Arc::clone(&self.state) }
    }
    fn clone_from(&mut self, source: &Self) {
        self.state = Arc::clone(&source.state)
    }
}
