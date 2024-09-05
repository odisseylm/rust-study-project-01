use std::sync::Arc;
use log::info;
use tonic::transport::Server;
use tonic_types::pb;
use mvv_common::cfg::ServerConf;
use mvv_common::env::process_env_load_res;
use mvv_common::exe::{current_exe_dir, current_exe_name};
use crate::cfg::ClientSearchSoaServerConfig;
use crate::dependencies::create_dependencies;
use crate::generated::mvv_client_search_api_v1::client_search_service_server::ClientSearchServiceServer;
use crate::server::{ClientSearchService};
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

    let serv_impl = Arc::new(ClientSearchService { dependencies });

    let client_search_serv = ClientSearchServiceServer::from_arc(serv_impl);

    use crate::generated::grpc_health_v1::health_server::HealthServer;
    let health_check_serv = HealthServer::new(HealthCheckService);

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(pb::FILE_DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET_334)
        .build_v1() ?;

    // TODO: add authentication/authorization
    Server::builder()
        .add_service(reflection_service)
        .add_service(client_search_serv)
        .add_service(health_check_serv)
        .serve(addr).await?;

    Ok(())
}
