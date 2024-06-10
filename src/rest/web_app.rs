use std::sync::Arc;
use axum::Router;
use crate::database::DatabaseConnection;
use crate::rest::account_rest::{ AccountRest, accounts_rest_router };
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

    // // Set environment for logging configuration
    // if std::env::var("RUST_LOG").is_err() {
    //     std::env::set_var("RUST_LOG", "info,myapp=debug");
    // }

    // env_logger::init();
    // env_logger::builder()
    //     .filter(None, log::LevelFilter::Info)
    //     .init();

    // tracing_subscriber::fmt()
    //     // .with_timer() /TODO: play with it
    //     // .with_env_filter(EnvFilter::from_default_env())
    //     .init();

    // tracing-subscriber which will REPLACE the env_logger
    //

    // tracing_subscriber::fmt::init();

    // Set environment for logging configuration
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::layer::SubscriberExt;
    use tracing_subscriber::util::SubscriberInitExt;

    // Start logging to console
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::Layer::default().compact())
        .init();

    log::info!("Hello from [web_app_main]");

    let dependencies = create_prod_dependencies();

    let app_router = Router::new()
        .merge(accounts_rest_router::<AccountServiceImpl>(dependencies.clone()));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app_router).await.unwrap();
}


/*
fn aaa() {
    use tracing::*;
    use tracing_timing::{Builder, Histogram};

    let subscriber = Builder::default().build(|| Histogram::new_with_max(1_000_000, 2).unwrap());
    let dispatcher = Dispatch::new(subscriber);
    dispatcher::with_default(&dispatcher, || {
        trace_span!("request").in_scope(|| {
            // do a little bit of work
            trace!("fast");
            // do a lot of work
            trace!("slow");
        })
    });
}
*/
