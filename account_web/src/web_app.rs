use std::sync::Arc;
use axum::Router;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use log::{error, info};

use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use mvv_auth::PswHashComparator;

use mvv_common::{
    db::pg::pg_db_connection,
    env::process_env_load_res,
    exe::{current_exe_name, current_exe_dir},
    rest::health_check_router,
    server::start_axum_server,
    utoipa::to_generate_open_api,
    net::ConnectionType,
};
use mvv_common::cfg::{BaseDependencyConnectConf, DependencyConnectConf, DependencyType};
use crate::{
    app_dependencies::{ Dependencies, DependenciesState },
    auth::SqlClientAuthUserProvider,
    cfg::{AccountWebServerConfig},
    service::account_service::{AccountServiceImpl, create_account_service},
};
use crate::service::client_search_service::{
    ClientSearchServiceImpl, create_client_search_service,
};
//--------------------------------------------------------------------------------------------------



fn create_prod_dependencies() -> Result<Arc<Dependencies>, anyhow::Error> {

    let db = Arc::new(pg_db_connection("account_web", ConnectionType::Ssl) ?);

    let account_soa_cfg = BaseDependencyConnectConf::load_from_env(
        "ACCOUNT_SOA".into(), DependencyType::REST, "account_web".into(),
    ) ?;
    let account_soa_cfg = account_soa_cfg.preload_values() ?;

    let client_search_soa_cfg = BaseDependencyConnectConf::load_from_env(
        "CLIENT_SEARCH_SOA".into(), DependencyType::GRPC, "account_web".into(),
    ) ?;
    let client_search_soa_cfg = client_search_soa_cfg.preload_values() ?;

    let account_service: Arc<AccountServiceImpl> = Arc::new(create_account_service(&account_soa_cfg) ?);
    let client_search_service: Arc<ClientSearchServiceImpl> = Arc::new(create_client_search_service(&client_search_soa_cfg) ?);

    Ok(Arc::new(Dependencies { state: Arc::new(DependenciesState {
        psw_comp: Arc::new(PswHashComparator::new()), // PlainPasswordComparator::new()),
        database_connection: Arc::clone(&db),
        account_service: Arc::clone(&account_service),
        client_search_service: Arc::clone(&client_search_service),
        user_perm_provider: Arc::new(SqlClientAuthUserProvider::with_cache(Arc::clone(&db)) ?)
    })}))
}

//noinspection DuplicatedCode
fn init_logger() {

    // Set environment for logging configuration
    // if std::env::var("RUST_LOG").is_err() {
    //     std::env::set_var("RUST_LOG", "info");
    // }

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
async fn create_app_route (dependencies: Arc<Dependencies>) -> Result<Router<()>, anyhow::Error> {

    use crate::auth::{ composite_auth_manager_layer, composite_login_router };

    let auth_layer = composite_auth_manager_layer(
        dependencies.state.psw_comp.clone(),
        Arc::clone(&dependencies.state.user_perm_provider),
    ).await ?;
    let login_route = composite_login_router();

    let app_router = Router::new()
        .merge(health_check_router())
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", create_open_api()))
        .merge(login_route)
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
                )
                .layer(auth_layer)
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
    info!("Hello from [web_app_main]");

    // !!! After logger initialization !!!
    process_env_load_res(&env_filename, dotenv_res) ?;

    if to_generate_open_api() {
        // generate_open_api(&create_open_api(), env!("CARGO_PKG_NAME"), UpdateApiFile::IfModelChanged, None) ?;
        return Ok(())
    }

    // let port = get_server_port("ACCOUNT_WEB") ?;
    let app_router = create_app_route(create_prod_dependencies() ?).await ?;
    let server_conf = AccountWebServerConfig::load_from_env() ?;

    start_axum_server(server_conf, app_router).await ?;

    Ok(())
}
