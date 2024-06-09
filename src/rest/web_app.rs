use std::sync::Arc;
use axum::Router;
use crate::database::DatabaseConnection;
use crate::rest::account_rest::{AccountRest, accounts_rest_router};
use crate::rest::app_dependencies::Dependencies;
use crate::service::account_service::AccountServiceImpl;


fn create_prod_dependencies() -> Dependencies<AccountServiceImpl> {

    let db = Arc::new(DatabaseConnection{});
    let account_service = Arc::new(AccountServiceImpl { database_connection: Arc::clone(&db) });
    let account_rest = Arc::new(AccountRest::<AccountServiceImpl> { account_service: Arc::clone(&account_service) });

    Dependencies::<AccountServiceImpl> {
        database_connection: Arc::clone(&db),
        account_service: Arc::clone(&account_service),
        account_rest: Arc::clone(&account_rest),
    }
}


pub async fn web_app_main() {

    let dependencies = create_prod_dependencies();

    let app_router = Router::new()
        .merge(accounts_rest_router::<AccountServiceImpl>(dependencies.clone()));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app_router).await.unwrap();
}



