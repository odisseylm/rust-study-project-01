use std::collections::HashSet;
use std::sync::Arc;
use log::info;
use tonic::{Request, Status};
// use tonic::service::interceptor::InterceptorLayer;
use tonic::transport::Server;
// use tonic_async_interceptor::AsyncInterceptedService;
use tonic_types::pb;
use tower::BoxError;
use tower::filter::FilterLayer;
use mvv_auth::{PlainPasswordComparator};
use mvv_common::cfg::ServerConf;
use mvv_common::env::process_env_load_res;
use mvv_common::exe::{current_exe_dir, current_exe_name};
// use mvv_common::rest::health_check_router;
use crate::auth::{AuthUser, CompositeAuthBackend, /*Role, RolePermissionsSet*/};
use crate::cfg::ClientSearchSoaServerConfig;
use crate::dependencies::{create_dependencies};
use crate::generated::mvv_client_search_api_v1::client_search_service_server::ClientSearchServiceServer;
use crate::grpc_auth::{grpc_req_enrich, GrpcAuthInterceptor};
use crate::server::ClientSearchService;
use crate::health_check::HealthCheckService;
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
pub const FILE_DESCRIPTOR_SET_334: &[u8] = tonic::include_file_descriptor_set!("mvv_client_search_descriptor");


/// This function will get called on each inbound request, if a `Status`
/// is returned, it will cancel the request and return that status to the
/// client.
fn intercept(req: Request<()>) -> Result<Request<()>, Status> {
    println!("Intercepting request: {:?}", req);

    // // Set an extension that can be retrieved by `say_hello`
    // req.extensions_mut().insert(MyExtension {
    //     some_piece_of_data: "foo".to_string(),
    // });

    Ok(req)
}

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



// #[tokio::main]
pub async fn grpc_app_main() -> Result<(), Box<dyn std::error::Error>> {

    let env_filename = format!(".{}.env", current_exe_name() ?);

    std::env::set_var("EXE_PATH_DIR", current_exe_dir() ?);
    let dotenv_res = dotenv::from_filename(&env_filename);

    init_logger();
    info!("Hello from [grpc_app_main]");

    // !!! After logger initialization !!!
    process_env_load_res(&env_filename, dotenv_res) ?;

    let conf = ClientSearchSoaServerConfig::load_from_env() ?;

    // let addr = std::env::var("GRPC_SERVER_ADDRESS") ?.parse()?;
    let addr = format!("0.0.0.0:{}", conf.server_port());
    let addr = addr.parse() ?;

    // run_migrations(&pool);

    let dependencies = Arc::new(create_dependencies() ?);

    use tonic_async_interceptor::async_interceptor;

    let grpc_auth_interceptor = GrpcAuthInterceptor::<AuthUser, /*Role, RolePermissionsSet,*/ CompositeAuthBackend> {
        public_endpoints: HashSet::from([
            // Using full endpoint names including method
            // "/grpc.reflection.v1.ServerReflection/ServerReflectionInfo".into(),
            // "/grpc.health.v1.Health/Check".into(),
            // "/grpc_health_v1.Health/Check".into(),
            // Or using full endpoint service names (without method)
            "/grpc.reflection.v1.ServerReflection".into(),
            "/grpc.health.v1.Health".into(),
            "/grpc_health_v1.Health".into(),
        ]),
        // user_provider: dependencies.user_provider.clone(),
        // permission_provider: dependencies.permission_provider.clone(),
        auth: CompositeAuthBackend::new_2(
            Arc::new(PlainPasswordComparator::new()),
            dependencies.user_provider.clone(),
            dependencies.permission_provider.clone(),
        ) ?,
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

    let client_search_serv =
        ClientSearchServiceServer::with_interceptor(ClientSearchService { dependencies: dependencies.clone() }, intercept)
        ;

    use crate::generated::grpc_health_v1::health_server::HealthServer;
    let health_check_serv = HealthServer::new(HealthCheckService);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(pb::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET_334)
        .build_v1() ?;

    /*
    let rest_route = tonic::service::Routes::from(health_check_router());
    use tower::timeout::TimeoutLayer;
    use std::time::Duration;

    use tower::ServiceBuilder;
    use tonic::{Request, Status, service::interceptor};

    let layer = ServiceBuilder::new()
        .load_shed()
        .timeout(Duration::from_secs(30))
        .layer(interceptor(auth_interceptor))
        .layer(interceptor(some_other_interceptor))
        // .layer(interceptor(some_other_interceptor_22))
        .into_inner();
    */

    // TODO: add authentication/authorization
    Server::builder()
        .layer(FilterLayer::new(grpc_req_enrich))
        // As example
        // .layer(tonic::service::interceptor(DependenciesSetInterceptor { dependencies: dependencies.clone() }))
        //
        .layer(async_interceptor(grpc_auth_interceptor))
        // .add_routes(rest_route)
        .add_service(reflection_service)
        .add_service(client_search_serv)
        .add_service(health_check_serv)
        .serve(addr)
        .await ?;

    Ok(())
}
