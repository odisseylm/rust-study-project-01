use std::collections::HashMap;
use std::sync::Arc;
use log::info;
use tonic::{
    transport::Server,
};
use tonic_types::pb;
use mvv_auth::PlainPasswordComparator;
use mvv_common::{
    cfg::ServerConf,
    env::process_env_load_res,
    exe::{current_exe_dir, current_exe_name},
    proto_files::{extract_proto_files, to_extract_proto_files},
    gen_src::UpdateFile,
};
// use mvv_common::rest::health_check_router;
use crate::{
    auth::{AuthUser, CompositeAuthBackend},
    cfg::ClientSearchSoaServerConfig,
    dependencies::{create_dependencies},
    grpc_auth::{GrpcAuthzInterceptor, TonicServerGrpcReqEnrichExt},
    client_search_service::ClientSearchService,
    health_check::HealthCheckService,
};
use crate::grpc::mvv::client::search::api::v1::client_search_service_server::ClientSearchServiceServer;
use crate::grpc_auth::predefined_public_endpoints_roles;
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

// pub const FILE_DESCRIPTOR_SET_333: &[u8] = include_bytes!("generated/types.bin");
pub const APP_SERVICES_FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("mvv_client_search_descriptor");


/*
#[derive(Debug, Clone)]
struct DependenciesContext {
    dependencies: Arc<Dependencies>,
}


#[derive(Debug, Clone)]
struct DependenciesSetInterceptor {
    dependencies: Arc<Dependencies>,
}
impl tonic::service::Interceptor for DependenciesSetInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        request.extensions_mut().insert(DependenciesContext {
            dependencies: self.dependencies.clone()
        });
        Ok(request)
    }
}
*/

#[derive(rust_embed::Embed)]
#[folder = "proto"]
pub struct ProtoFiles;


// #[tokio::main]
pub async fn grpc_app_main() -> Result<(), Box<dyn std::error::Error>> {

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
    let addr = addr.parse() ?;

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
            Arc::new(PlainPasswordComparator::new()),
            dependencies.user_provider.clone(),
            dependencies.permission_provider.clone(),
        ) ?),
    };

    /*
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
    Server::builder()
        .add_grpc_req_enrich_layer() // similar to `.layer(FilterLayer::new(grpc_req_enrich))`
        // As example
        // .layer(tonic::service::interceptor(DependenciesSetInterceptor { dependencies: dependencies.clone() }))
        //
        .layer(async_interceptor(grpc_auth_interceptor))
        // .add_routes(rest_route)
        .add_service(reflection_service)
        .add_service(health_check_serv)
        .add_service(client_search_serv)
        .serve(addr)
        .await ?;


    Ok(())
}
