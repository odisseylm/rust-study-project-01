use std::sync::Arc;
use axum::Router;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use log::{ error /*, info*/ };

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use mvv_auth::PlainPasswordComparator;

use mvv_common::{
    env::process_env_load_res,
    exe::{current_exe_name, current_exe_dir},
    rest::health_check_router,
    server::start_axum_server,
    utoipa::{generate_open_api, nest_open_api, to_generate_open_api, UpdateApiFile},
};
use crate::cfg::AccountSoaServerConfig;
use crate::service::{
    account_service::{ AccountService, AccountServiceImpl },
};
use crate::rest::{
    app_dependencies::{ Dependencies, DependenciesState },
    account_rest::{ AccountRest, accounts_rest_router, AccountRestOpenApi },
    auth::user_perm_provider::SqlUserProvider,
};
use crate::web::{
    auth::composite_login_router,
    templates::protected_page_01::protected_page_01_router,
};
//--------------------------------------------------------------------------------------------------



fn create_prod_dependencies() -> Result<Dependencies<AccountServiceImpl>, anyhow::Error> {

    let db = Arc::new(mvv_common::db::pg::pg_db_ssl_connection("account_soa") ?);
    let account_service = Arc::new(AccountServiceImpl { database_connection: Arc::clone(&db) });
    let account_rest = Arc::new(AccountRest::<AccountServiceImpl> { account_service: Arc::clone(&account_service) });

    Ok(Dependencies::<AccountServiceImpl> { state: Arc::new(DependenciesState {
        database_connection: Arc::clone(&db),
        account_service: Arc::clone(&account_service),
        account_rest: Arc::clone(&account_rest),
        psw_comparator: Arc::new(PlainPasswordComparator::new()),
        user_perm_provider: Arc::new(SqlUserProvider::with_cache(Arc::clone(&db)) ?)
    })})
}

//noinspection DuplicatedCode
fn init_logger() {

    // Set environment for logging configuration
    // if std::env::var("RUST_LOG").is_err() {
    //     std::env::set_var("RUST_LOG", "info");
    // }

    // env_logger::init();
    // env_logger::builder()
    //     .filter(None, log::LevelFilter::Info)
    //     .init();

    use tracing_subscriber::{
        EnvFilter, layer::SubscriberExt, util::SubscriberInitExt,
    };

    // Start logging to console
    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,example_error_handling=debug,tower_http=debug,validator=debug,validify=debug,axum_valid=debug".into()),
        )
        //.with(EnvFilter::from_default_env())
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


#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "mvv_account_soa", description = "Rust account SOA")
    )
)]
struct RootOpenApi;

fn create_open_api() -> utoipa::openapi::OpenApi {
    let mut root_open_api = RootOpenApi::openapi();
    root_open_api.merge(nest_open_api("/api", &AccountRestOpenApi::openapi()));
    root_open_api
}


//noinspection DuplicatedCode
async fn create_app_route <
        AccountS: AccountService + Send + Sync + 'static,
        // AccountR: AccountRest<AccountS> + Send + Sync,
    >
    (dependencies: Dependencies<AccountS>) -> Result<Router<()>, anyhow::Error> {

    use crate::rest::auth::auth_layer::{ composite_auth_manager_layer };
    let auth_layer =
        composite_auth_manager_layer(
            // Arc::clone(&dependencies.state.psw_comparator),
            dependencies.state.psw_comparator.clone(),
            Arc::clone(&dependencies.state.user_perm_provider),
        ).await ?;
    let login_route = composite_login_router();

    let app_router = Router::new()
        .merge(health_check_router())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", create_open_api()))
        .merge(login_route)
        .merge(protected_page_01_router())
        .nest("/api", Router::new()
            .merge(accounts_rest_router::<AccountS>(dependencies.clone()))
            .nest("/admin", Router::new()
                  // .merge()
            )
        )
        .layer(
            ServiceBuilder::new()
                .map_err(|err| {
                    error!("### error: {:?}", err);
                    err
                })
                .layer(TraceLayer::new_for_http()
                /*
                // From https://github.com/tokio-rs/axum/blob/main/examples/error-handling/src/main.rs
                //
                //
                // Create our own span for the request and include the matched path. The matched
                // path is useful for figuring out which handler the request was routed to.
                .make_span_with(|req: &Request| {
                    let method = req.method();
                    let uri = req.uri();

                    // axum automatically adds this extension.
                    let matched_path = req
                        .extensions()
                        .get::<MatchedPath>()
                        .map(|matched_path| matched_path.as_str());

                    tracing::debug_span!("request", %method, %uri, matched_path)
                })
                // By default `TraceLayer` will log 5xx responses but we're doing our specific
                // logging of errors so disable that
                .on_failure(())
                */
                )
                .layer(auth_layer)
                // additional state which will/can be accessible for ALL route methods
                // .layer(Extension(Arc::new(State22 { x: "963" })))
                .map_err(|err|{
                    error!("### Route error: {:?}", err); err
                })
        );
    Ok(app_router)
}


pub async fn web_app_main() -> Result<(), anyhow::Error> {

    let env_filename = format!(".{}.env", current_exe_name() ?);

    std::env::set_var("EXE_PATH_DIR", current_exe_dir() ?);
    let dotenv_res = dotenv::from_filename(&env_filename);

    init_logger();
    log::info!("Hello from [web_app_main]");

    // !!! After logger initialization !!!
    process_env_load_res(&env_filename, dotenv_res) ?;

    if to_generate_open_api() {
        generate_open_api(&create_open_api(), env!("CARGO_PKG_NAME"), UpdateApiFile::IfModelChanged, None) ?;
        return Ok(())
    }

    let app_router = create_app_route(create_prod_dependencies() ?).await ?;
    let server_cfg = AccountSoaServerConfig::load_from_env("account_soa".into(), "ACCOUNT_SOA".into()) ?;

    start_axum_server(server_cfg, app_router).await ?;

    Ok(())
}



/*
fn with_histogram() {
    use tracing::*;
    use tracing_timing::{ Builder, Histogram };

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
