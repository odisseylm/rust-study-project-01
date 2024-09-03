use std::sync::Arc;
use log::info;
use tonic::transport::Server;
use mvv_common::env::process_env_load_res;
use mvv_common::exe::{current_exe_dir, current_exe_name};
use crate::dependencies::create_dependencies;
use crate::generated::mvv_client_search_api_v1::client_search_service_server::ClientSearchServiceServer;
use crate::server::ClientSearchService;
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


// #[tokio::main]
pub async fn grpc_app_main() -> Result<(), Box<dyn std::error::Error>> {

    let env_filename = format!(".{}.env", current_exe_name() ?);

    std::env::set_var("EXE_PATH_DIR", current_exe_dir() ?);
    let dotenv_res = dotenv::from_filename(&env_filename);

    init_logger();
    info!("Hello from [grpc_app_main]");

    // !!! After logger initialization !!!
    process_env_load_res(&env_filename, dotenv_res) ?;

    let addr = std::env::var("GRPC_SERVER_ADDRESS") ?.parse()?;

    // let pool = create_connection_pool();
    // run_migrations(&pool);

    let dependencies = Arc::new(create_dependencies() ?);

    // let solar_system_info = SolarSystemInfoService { pool };
    // let svc = SolarSystemInfoServer::new(solar_system_info);
    let serv_impl = Arc::new(ClientSearchService { dependencies });

    // let server = crate::mvv_client_search_api_v1::client_search_service_server::
    let serv = ClientSearchServiceServer::from_arc(serv_impl);

    Server::builder().add_service(serv).serve(addr).await?;

    Ok(())
}
