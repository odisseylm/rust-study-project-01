use core::convert::Infallible;
use std::{
    collections::HashMap,
    error::Error as StdError,
    net::SocketAddr,
    sync::Arc,
};
use anyhow::anyhow;
use axum_server::service::SendService;
use log::{error, info};
use tonic::{
    transport::Server,
    service::interceptor,
    // transport::ServerTlsConfig,
};
use tonic_types::pb;
use tower::{
    BoxError, ServiceBuilder,
    filter::FilterLayer,
};
use mvv_auth::{
    AuthUserProvider, PasswordComparator, PlainPasswordComparator,
    grpc::{
        server::{GrpcAuthzInterceptor, predefined_public_endpoints_roles},
        server::{grpc_req_enrich, axum::axum_grpc_req_enrich},
    },
    permission::PermissionProvider,
};
use mvv_common::{
    cfg::{ServerConf, SslConfValue},
    env::process_env_load_res,
    exe::{current_exe_dir, current_exe_name},
    gen_src::UpdateFile,
    grpc::server::server_tls_config,
    net::ConnectionType,
    proto_files::{extract_proto_files, to_extract_proto_files},
    rest::health_check_router,
    server::{server_rustls_config, start_axum_server},
};
use crate::{
    auth::{AuthUser, CompositeAuthBackend, Role, RolePermissionsSet},
    cfg::ClientSearchSoaServerConfig,
    dependencies::{create_dependencies},
    client_search_service::ClientSearchService,
    health_check::HealthCheckService,
};
use crate::grpc::mvv::client::search::api::v1::client_search_service_server::ClientSearchServiceServer;
//--------------------------------------------------------------------------------------------------



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

    /*
    tracing_subscriber::registry()
        .with(EnvFilter::new(std::env::var("RUST_LOG").unwrap_or_else(
            |_| "axum_login=debug,tower_sessions=debug,sqlx=warn,tower_http=debug".into(),
        )))
        .with(tracing_subscriber::fmt::layer())
        .try_init()?;
    */
}

pub const APP_SERVICES_FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("mvv_client_search_descriptor");



#[derive(rust_embed::Embed)]
#[folder = "proto"]
pub struct ProtoFiles;


pub async fn grpc_app_main() -> Result<(), Box<dyn StdError>> {

    let env_filename = format!(".{}.env", current_exe_name() ?);

    std::env::set_var("EXE_PATH_DIR", current_exe_dir() ?);
    let dotenv_res = dotenv::from_filename(&env_filename);

    init_logger();
    info!("Hello from [grpc_app_main]");

    // !!! After logger initialization !!!
    process_env_load_res(&env_filename, dotenv_res) ?;

    if to_extract_proto_files() {
        extract_proto_files::<ProtoFiles>( env!("CARGO_PKG_NAME"), UpdateFile::IfModelChanged, None) ?;
        return Ok(())
    }

    let conf = ClientSearchSoaServerConfig::load_from_env() ?;

    // let addr = std::env::var("GRPC_SERVER_ADDRESS") ?.parse()?;
    let addr = format!("0.0.0.0:{}", conf.server_port());
    let addr: SocketAddr = addr.parse() ?;

    // run_migrations(&pool);

    let dependencies = Arc::new(create_dependencies() ?);

    use tonic_async_interceptor::async_interceptor;

    let grpc_auth_interceptor = GrpcAuthzInterceptor::<AuthUser, /*Role, RolePermissionsSet,*/ CompositeAuthBackend> {
        endpoints_roles: Arc::new({
            let mut roles = HashMap::new();
            roles.extend(predefined_public_endpoints_roles());
            roles.extend(ClientSearchService::endpoints_roles());
            roles
        }),
        auth: Arc::new(CompositeAuthBackend::new_2(
            dependencies.password_comparator.clone(),
            dependencies.user_provider.clone(),
            dependencies.permission_provider.clone(),
        ) ?),
    };

    /*
    // With interceptor
    //
    let client_search_serv = AsyncInterceptedService::new(
        ClientSearchServiceServer::from_arc(serv_impl),
        grpc_auth_interceptor,
    );

    let client_search_serv = AsyncInterceptedService::new(
        client_search_serv,
        authenticate,
    );
    */

    // With interceptor
    //
    // let client_search_serv =
    //     ClientSearchServiceServer::with_interceptor(ClientSearchService { dependencies: dependencies.clone() }, intercept)
    //     ;

    let client_search_serv =
        ClientSearchServiceServer::new(ClientSearchService { dependencies: dependencies.clone() });

    use crate::grpc::health::v1::health_server::HealthServer;
    let health_check_serv = HealthServer::new(HealthCheckService);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(pb::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(APP_SERVICES_FILE_DESCRIPTOR_SET)
        .build_v1() ?;

    let routes = tonic::service::Routes::builder().routes()
        .add_service(reflection_service.clone())
        .add_service(health_check_serv.clone())
        .add_service(client_search_serv.clone())
        ;


    match conf.connection_type() {

        ConnectionType::Plain => {
            use mvv_auth::grpc::server::TonicServerGrpcReqEnrichExt;

            Server::builder()
                // !!! T O D O: At that moment it silently crashes with enabled SSL !!!
                // !!!          at least with rustls                                !!!
                //
                // .tls_config(server_tls_config(&conf).await ?) ? // now it crashes
                //

                // Other configurations/customizations
                // .concurrency_limit_per_connection(256)
                //

                // Authentication/authorization configuration
                //
                .add_grpc_req_enrich_layer()? // similar to `.layer(FilterLayer::new(grpc_req_enrich))`
                // As example
                // .layer(tonic::service::interceptor(DependenciesSetInterceptor { dependencies: dependencies.clone() }))
                //
                // .layer(async_interceptor(grpc_auth_interceptor))
                //
                .add_grpc_auth_layer(grpc_auth_interceptor)?

                // .add_routes(rest_route)
                .add_routes(routes)
                .serve(addr)
                .await ?
        }

        ConnectionType::Ssl => {
            // !!! WORKAROUND !!!:
            //  At that moment tonic server silently crashes with enabled SSL.
            //  As workaround now I use axum/axum-server for SSL mode.
            //  Use tonic when this bug is fixed.

            let app_axum_router = axum::Router::new()
                .merge(health_check_router())
                .merge(routes.into_axum_router())
                .layer(
                    ServiceBuilder::new()
                        .layer(tower_http::trace::TraceLayer::new_for_http())
                        // We cannot reuse tower FilterLayer/AsyncFilterLayer there
                        // (similar to tonic approach) since Error incompatibility
                        // between axum (Infallible) and tower BoxError
                        .layer(axum::middleware::from_fn(axum_grpc_req_enrich))
                        .layer(async_interceptor(grpc_auth_interceptor))
                );
            start_axum_server(conf, app_axum_router).await ?;
        }

        ConnectionType::Auto =>
            Err(anyhow!("Server connection type auto detection is not supported")) ?,
    }

    Ok(())
}
