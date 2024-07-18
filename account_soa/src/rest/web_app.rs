use std::sync::Arc;
use anyhow::anyhow;
// use axum::error_handling::{ HandleError, HandleErrorLayer };
use axum::Router;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

// use utoipa::openapi::{ OpenApiBuilder, PathItem, PathsBuilder };
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::service::account_service::AccountServiceImpl;
use crate::rest::{
    app_dependencies::Dependencies,
    account_rest::{ AccountRest, accounts_rest_router },
};
use crate::rest::auth::user_perm_provider::SqlUserProvider;
use mvv_common::env::env_var;
use mvv_common::string::remove_optional_suffix;
use crate::web::{
    auth::composite_login_router,
    templates::protected_page_01::protected_page_01_router,
};
//--------------------------------------------------------------------------------------------------



fn create_prod_dependencies() -> Result<Dependencies<AccountServiceImpl>, anyhow::Error> {

    let db = Arc::new(mvv_common::db::pg::pg_db_connection() ?);
    let account_service = Arc::new(AccountServiceImpl { database_connection: Arc::clone(&db) });
    let account_rest = Arc::new(AccountRest::<AccountServiceImpl> { account_service: Arc::clone(&account_service) });

    Ok(Dependencies::<AccountServiceImpl> { state: Arc::new(DependenciesState {
        database_connection: Arc::clone(&db),
        account_service: Arc::clone(&account_service),
        account_rest: Arc::clone(&account_rest),
        user_perm_provider: Arc::new(SqlUserProvider::with_cache(Arc::clone(&db)) ?)
    })})
}

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

pub async fn web_app_main() -> Result<(), anyhow::Error> {
    use core::str::FromStr;

    let env_filename = format!(".{}.env", current_exe_name() ?);
    let dotenv_res = dotenv::from_filename(env_filename.as_str());

    let port_env = env_var("SERVER_PORT") ?;
    let port_env = port_env.as_deref().unwrap_or("3000");
    let port: u32 = FromStr::from_str(port_env)
        .map_err(|_|anyhow!("SERVER_PORT value [{}] has wrong format.", port_env)) ?;

    init_logger();
    log::info!("Hello from [web_app_main]");

    match dotenv_res { // TODO: move it out
        Ok(ref path) =>
            log::info!("Env vars are loaded from [{:?}]", path),
        Err(dotenv::Error::Io(ref io_err))
            if io_err.kind() == std::io::ErrorKind::NotFound => {
            log::info!("Env vars are not loaded from [{env_filename}] file.");
        }
        Err(ref _err) => {
            log::error!("Error of loading .env file.");
            anyhow::bail!("Error of loading .env file.");
        }
    }

    #[derive(OpenApi)]
    #[openapi( )]
    struct RootOpenApi;

    /*
    #[derive(OpenApi)]
    #[openapi()]
    struct HelloApi;

    let hello_api =
        Into::<OpenApiBuilder>::into(HelloApi::openapi()).paths(PathsBuilder::new().path(
            // "/api/client/00000000-0000-0000-0000-000000000001/account/all",
            "/qwerty/api/client/${client_id}/account/all",
            PathItem::new(utoipa::openapi::PathItemType::Get, Operation::new()),
        ));
    */

    let mut root_open_api = RootOpenApi::openapi();

    // let hello_api2 =
    //     Into::<OpenApiBuilder>::into(ApiDoc2::openapi());
    // hello_api2.

    root_open_api.merge(nest_open_api("/api", &AccountRestOpenApi::openapi()));

    // let mut open_api = ApiDoc::openapi();
    // open_api.merge(hello_api.build());
    // open_api = open_api.nest("/hello", hello_api); // you can even nest programmatically apis
    // open_api.merge(hello_api.build()); // you can even nest programmatically apis

    let dependencies = create_prod_dependencies() ?;

    use crate::rest::auth::auth_layer::{ composite_auth_manager_layer };
    let auth_layer =
        composite_auth_manager_layer(Arc::clone(&dependencies.state.user_perm_provider)).await ?;
    let login_route = composite_login_router();

    let app_router = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", root_open_api))
        // .merge(SwaggerUi::new("/swagger-ui").url("/api-doc/openapi.json", ApiDoc::openapi()))
        // .route_service("/", HandleError::new(fallible_service, handle_error) )
        .merge(login_route)
        .merge(protected_page_01_router())
        .nest("/api", Router::new()
            .merge(accounts_rest_router::<AccountServiceImpl>(dependencies.clone()))
            /*
            .nest("/current_user", Router::new()
                // .merge(accounts_rest_router::<AccountServiceImpl>(dependencies.clone()))
                // .merge(user details router)
                // Other 'current user' services
            )
            */
            .nest("/admin", Router::new()
                // .merge()
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
                //.layer(HandleErrorLayer::new(handle_error))
                //.timeout(Duration::from_secs(10))
                // .map_err(|err: validator::ValidationError| {
                .map_err(|err| {
                    error!("### error 455: {:?}", err);
                    err
                })
                // .map_result(|result: Result<Self::Response, Self::Error>| match result {
                /*
                .map_result(|result: Result<_, _>| match result {
                    // If the error indicates that no records matched the query, return an empty
                    // `Vec` instead.
                    // Err(DbError::NoRecordsFound) => Ok(Vec::new()),
                    // Propagate all other responses (`Ok` and `Err`) unchanged

                    // x => x,
                    Ok(ref _ok) => {
                        let temp: &axum::response::Response = _ok;
                        info!("### error 456: {:?}", _ok);
                        if _ok.status() != 200 {

                        }
                        result},
                    Err(ref _err) =>
                        result,
                })
                */
                /*
                .map_result(|result: Result<_, validator::ValidationError>| match result {
                    // If the error indicates that no records matched the query, return an empty
                    // `Vec` instead.
                    // Err(DbError::NoRecordsFound) => Ok(Vec::new()),
                    // Propagate all other responses (`Ok` and `Err`) unchanged

                    // x => x,
                    Ok(ref _ok) => {
                        // let temp: &axum::response::Response = _ok;
                        // info!("### error 456: {:?}", _ok);
                        // if _ok.status() != 200 {
                        //
                        // }
                        result},
                    Err(ref _err) =>
                        result,
                })
                */
        );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await ?;
    axum::serve(listener, app_router).await ?;
    Ok(())
}


// use axum::{
//     // BoxError,
//     // response::IntoResponse,
//     // http::{StatusCode, Method, Uri},
//     // routing::get,
// };
// use tower::{ timeout::error::Elapsed};
use log::{ error /*, info*/ };
use mvv_common::utoipa::nest_open_api;
use crate::rest::account_rest::AccountRestOpenApi;
use crate::rest::app_dependencies::DependenciesState;
// use axum_handle_error_extract::HandleErrorLayer;

/*
async fn handle_error_2() -> Result<String, StatusCode> {
    // ...
    t o d o!()
}


async fn handle_error(
    method: Method,
    uri: Uri,
    error: BoxError,
) -> impl IntoResponse {
    if error.is::<Elapsed>() {
        (
            StatusCode::REQUEST_TIMEOUT,
            format!("{} {} took too long", method, uri),
        )
    } else {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("{} {} failed: {}", method, uri, error),
        )
    }
}
*/


// Returns without extension
fn current_exe_name() -> Result<String, anyhow::Error> {
    let cur_exe_as_os_str = std::env::current_exe()
        .map(|ref p| p.file_name().map(|s|s.to_os_string())) ?
        .ok_or_else(||anyhow!("Cannot find executable name.")) ?;
    let cur_exe = cur_exe_as_os_str.to_string_lossy().to_string();
    let cur_exe = remove_optional_suffix(cur_exe, ".exe");
    Ok(cur_exe)
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
