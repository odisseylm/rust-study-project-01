use core::time::Duration;
use std::net::SocketAddr;
use log::info;
use tokio::signal;
use crate::{
    cfg::{ServerConf, SslConfValue},
    client_cert_auth::server_rustls_with_ssl_cert_client_auth_config,
    net::ConnectionType,
    rustls_acceptor_with_con_info::{ServiceWrapperRustlsAcceptor, ServiceWrapperExt},
};
//--------------------------------------------------------------------------------------------------


pub async fn start_axum_server<Conf: ServerConf>(server_conf: Conf, app_router: axum::routing::Router) -> anyhow::Result<()> {

    let connection_type = server_conf.connection_type();
    let port = server_conf.server_port();
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let handle = axum_server::Handle::new();
    tokio::spawn(axum_server_shutdown_signal(
        handle.clone(), server_conf.shutdown_timeout()));

    match connection_type {
        ConnectionType::Plain => {

            // Using axum-server third-party crate
            info!("Web server started on plain port [{port}]");
            axum_server::bind(addr)
                .handle(handle)
                .serve(app_router.into_make_service())
                .await ?;

            // Using axum core
            /*
            // run our app with hyper, listening globally on port 3000
            let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await ?;
            info!("Web server started on port [{port}]");
            axum::serve(listener, app_router)
                // T O D O: how to use shutdown timeout with this axum core?
                .with_graceful_shutdown(axum_core_shutdown_signal()) ???
                .await ?;
            */
        }

        ConnectionType::Ssl => {

            let rust_tls_config =
                if server_conf.client_auth_ssl_ca_cert().is_some() {
                    server_rustls_with_ssl_cert_client_auth_config(&server_conf).await ?
                } else {
                    server_rustls_config(&server_conf).await ?
                };

            info!("Web server started on ssl port [{port}]");
            // axum_server::bind_rustls(addr, rust_tls_config)
            //
            // For possibility to use client cert authentication
            axum_server_bind_rustls_with_con_info(addr, rust_tls_config)
                .handle(handle)
                // .serve(app_router.into_make_service())
                //
                // For possibility to use client cert authentication
                .serve(app_router.into_make_service_with_con_info())
                .await ?;
        }

        ConnectionType::Auto =>
            anyhow::bail!("Server connection type auto detection is not supported"),
    }

    Ok(())
}



pub fn axum_server_bind_rustls_with_con_info(
    addr: SocketAddr, config: axum_server::tls_rustls::RustlsConfig)
    -> axum_server::Server<ServiceWrapperRustlsAcceptor> {

    use axum_server::Server;

    let acceptor: ServiceWrapperRustlsAcceptor = ServiceWrapperRustlsAcceptor::new(config)
        //.acceptor(MyDefaultAcceptor) // for additional TcpStream customization
        ;

    let server: Server<ServiceWrapperRustlsAcceptor> =
        Server::bind(addr).acceptor(acceptor);
    server
}



pub async fn server_rustls_config<Conf: ServerConf>(server_conf: &Conf)
    -> anyhow::Result<axum_server::tls_rustls::RustlsConfig> {

    use axum_server::tls_rustls::RustlsConfig;

    let server_name = server_conf.server_name();
    let server_name = server_name.as_str();

    if let (Some(SslConfValue::Path(server_key)), Some(SslConfValue::Path(server_cert))) =
        (&server_conf.server_ssl_key(), &server_conf.server_ssl_cert()) {

        Ok(RustlsConfig::from_pem_file(server_cert, server_key).await ?)

    } else if let (Some(SslConfValue::Value(server_key)), Some(SslConfValue::Value(server_cert))) =
        (&server_conf.server_ssl_key(), &server_conf.server_ssl_cert()) {

        Ok(RustlsConfig::from_pem(
            Vec::from(server_cert.as_bytes()),
            Vec::from(server_key.as_bytes()),
        ).await ?)
    } else {
        anyhow::bail!("Both {server_name} cert/key should have the same type")
    }
}


// See
// * axum core: https://github.com/tokio-rs/axum/blob/main/examples/graceful-shutdown/src/main.rs
// * axum_server: ...
pub async fn axum_server_shutdown_signal(
    handle: axum_server::Handle, max_shutdown_duration: Duration) {

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("graceful_shutdown (ctrl_c)");
            handle.graceful_shutdown(Some(max_shutdown_duration))
        },
        _ = terminate => {
            info!("graceful_shutdown (terminate)");
            handle.graceful_shutdown(Some(max_shutdown_duration))
        },
    }
}
