use std::sync::Arc;
use axum::Router;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use log::{error, info};

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use mvv_auth::PswHashComparator;

use mvv_common::{
    env::process_env_load_res,
    exe::current_exe_name,
    server_conf::get_server_port,
    // utoipa::{ generate_open_api, nest_open_api, to_generate_open_api, UpdateApiFile },
};
use mvv_common::cfg::SslConfValue;
use mvv_common::utoipa::to_generate_open_api;
use crate::{
    app_dependencies::{ Dependencies, DependenciesState },
    service::account_service::{AccountService, AccountServiceImpl },
    // auth::in_mem_client_auth_user_provider,
};
use crate::auth::SqlClientAuthUserProvider;
use crate::cfg::SslConfig;
use crate::service::account_service::{AccountSoaConnectCfg, create_account_service};
// use crate::web::{
//     auth::composite_login_router,
//     templates::protected_page_01::protected_page_01_router,
// };
//--------------------------------------------------------------------------------------------------



fn create_prod_dependencies() -> Result<Arc<Dependencies<AccountServiceImpl>>, anyhow::Error> {

    let db = Arc::new(mvv_common::db::pg::pg_db_connection("account_web") ?);

    let account_soa_cfg = AccountSoaConnectCfg::load_from_env() ?;

    let account_service: Arc<AccountServiceImpl> = Arc::new(create_account_service(&account_soa_cfg) ?);
    //let account_rest = Arc::new(AccountRest::<AccountServiceImpl> { account_service: Arc::clone(&account_service) });

    Ok(Arc::new(Dependencies::<AccountServiceImpl> { state: Arc::new(DependenciesState {
        psw_comp: Arc::new(PswHashComparator::new()), // PlainPasswordComparator::new()),
        database_connection: Arc::clone(&db),
        account_service: Arc::clone(&account_service),
        user_perm_provider: Arc::new(SqlClientAuthUserProvider::with_cache(Arc::clone(&db)) ?)
    })}))
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
    let root_open_api = RootOpenApi::openapi();
    // let mut root_open_api = RootOpenApi::openapi();
    // root_open_api.merge(nest_open_api("/api", &AccountRestOpenApi::openapi()));
    root_open_api
}


//noinspection DuplicatedCode
async fn create_app_route <
        AccountS: AccountService + Send + Sync + 'static,
        // AccountR: AccountRest<AccountS> + Send + Sync,
    >
    (dependencies: Arc<Dependencies<AccountS>>) -> Result<Router<()>, anyhow::Error> {

    // let dependencies = Arc::new(dependencies);

    use crate::auth::{ composite_auth_manager_layer, composite_login_router };

    let auth_layer = composite_auth_manager_layer(
        dependencies.state.psw_comp.clone(),
        Arc::clone(&dependencies.state.user_perm_provider),
    ).await ?;
    let login_route = composite_login_router();

    let app_router = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", create_open_api()))
        .merge(login_route)
        // .merge(protected_page_01_router())
        .nest("/api", Router::new()
            // .merge(accounts_rest_router::<AccountS>(dependencies.clone()))
            .nest("/admin", Router::new()
                  // .merge()
            )
        )
        .nest("/ui", Router::new()
            .merge(crate::mvc::action_client_accounts::accounts_web_router(dependencies.clone()))
            .merge(crate::mvc::action_client_accounts::current_client_accounts_router(dependencies.clone()))
            // .merge(accounts_rest_router::<AccountS>(dependencies.clone()))
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
    let dotenv_res = dotenv::from_filename(&env_filename);

    init_logger();
    info!("Hello from [web_app_main]");

    // !!! After logger initialization !!!
    process_env_load_res(&env_filename, dotenv_res) ?;

    if to_generate_open_api() {
        // generate_open_api(&create_open_api(), env!("CARGO_PKG_NAME"), UpdateApiFile::IfModelChanged, None) ?;
        return Ok(())
    }

    let port = get_server_port("ACCOUNT_WEB") ?;
    let app_router = create_app_route(create_prod_dependencies() ?).await ?;

    use axum_server::tls_rustls::RustlsConfig;
    let ssl_conf = SslConfig::load_from_env() ?;

    let rust_tls_config: RustlsConfig =
        if let (SslConfValue::Path(account_web_cert), SslConfValue::Path(account_web_key)) =
                                   (&ssl_conf.account_web_cert, &ssl_conf.account_web_key) {
            RustlsConfig::from_pem_file(account_web_cert, account_web_key).await ?
        } else if let (SslConfValue::Value(account_web_cert), SslConfValue::Value(account_web_key)) =
                                   (&ssl_conf.account_web_cert, &ssl_conf.account_web_key) {
            RustlsConfig::from_pem(
                Vec::from(account_web_cert.as_bytes()),
                Vec::from(account_web_key.as_bytes()),
            ).await ?
        } else {
            anyhow::bail!("Both account_soa_cert/account_soa_key should have the same type")
        };

    // // run our app with hyper, listening globally on port 3001
    // let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await ?;
    // info!("Web server started on port [{port}]");
    // axum::serve(listener, app_router).await ?;

    use std::net::SocketAddr;
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    axum_server::bind_rustls(addr, rust_tls_config)
        .serve(app_router.into_make_service())
        .await ?;

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
