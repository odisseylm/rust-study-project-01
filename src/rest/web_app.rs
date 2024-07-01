use std::sync::Arc;
use axum::Router;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::database::DatabaseConnection;
use crate::service::account_service::AccountServiceImpl;
use crate::rest::{
    app_dependencies::Dependencies,
    account_rest::{ CurrentUserAccountRest, accounts_rest_router },
};
use crate::util::env_var_or_else;
use crate::web::{
    auth::composite_login_router,
    templates::protected_page_01::protected_page_01_router,
};


fn create_prod_dependencies() -> Dependencies<AccountServiceImpl> {

    let db = Arc::new(DatabaseConnection{});
    let account_service = Arc::new(AccountServiceImpl { database_connection: Arc::clone(&db) });
    let account_rest = Arc::new(CurrentUserAccountRest::<AccountServiceImpl> { account_service: Arc::clone(&account_service) });

    Dependencies::<AccountServiceImpl> {
        database_connection: Arc::clone(&db),
        account_service: Arc::clone(&account_service),
        account_rest: Arc::clone(&account_rest),
    }
}

fn init_logger() {
    // // Set environment for logging configuration
    // if std::env::var("RUST_LOG").is_err() {
    //     std::env::set_var("RUST_LOG", "info,myapp=debug");
    // }

    // env_logger::init();
    // env_logger::builder()
    //     .filter(None, log::LevelFilter::Info)
    //     .init();

    // tracing_subscriber::fmt()
    //     // .with_timer() // T O D O: play with it
    //     // .with_env_filter(EnvFilter::from_default_env())
    //     .init();

    // tracing-subscriber which will REPLACE the env_logger
    //

    // tracing_subscriber::fmt::init();

    // Set environment for logging configuration
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    use tracing_subscriber::{
        EnvFilter, layer::SubscriberExt, util::SubscriberInitExt,
    };

    // Start logging to console
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::Layer::default().compact())
        .init();

    /*
    tracing_subscriber::registry()
        .with(EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(
            |_| "axum_login=debug,tower_sessions=debug,sqlx=warn,tower_http=debug".into(),
        )))
        .with(tracing_subscriber::fmt::layer())
        .try_init()?;
    */
}

pub async fn web_app_main() -> Result<(), anyhow::Error> {
    use core::str::FromStr;

    let port_env = env_var_or_else("SERVER_PORT", || "3000".to_string()) ?;
    let port: u32 = FromStr::from_str(port_env.as_str()) ?;

    init_logger();
    log::info!("Hello from [web_app_main]");

    let dependencies = create_prod_dependencies();

    use crate::rest::auth::auth_layer::{ composite_auth_manager_layer };
    let auth_layer = composite_auth_manager_layer().await ?;
    let login_route = composite_login_router();

    // #[allow(non_upper_case_globals)]
    // const login_url_prefix: &'static str = ""; // "/mvv_auth_555/login_form";
    // use crate::rest::auth::{ login_form_auth_manager_layer, AuthUser, PswComparator };
    // use const_format::concatcp;
    // let auth_layer = login_form_auth_manager_layer(concatcp!(login_url_prefix, "/login")).await ?;
    // let login_route = Router::new().nest(login_url_prefix,
    //           mvv_auth::backend::login_form_auth::web::login_router::
    //             <AuthUser,PswComparator,Role,RolePermissionsSet>());

    let app_router = Router::new()
        .merge(login_route)
        .merge(protected_page_01_router())
        .merge(accounts_rest_router::<AccountServiceImpl>(dependencies.clone()))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(auth_layer)
                // additional state which will/can be accessible for ALL route methods
                // .layer(Extension(Arc::new(State22 { x: "963" })))
        );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await ?;
    axum::serve(listener, app_router).await ?;
    Ok(())
}


/*
fn with_histogram() {
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
