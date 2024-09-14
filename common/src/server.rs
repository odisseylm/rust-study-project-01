use log::info;
use crate::cfg::{ServerConf, SslConfValue};
use crate::net::ConnectionType;
//--------------------------------------------------------------------------------------------------


pub async fn start_axum_server<Conf: ServerConf>(server_conf: Conf, app_router: axum::routing::Router) -> anyhow::Result<()> {

    use axum_server::tls_rustls::RustlsConfig;

    let connection_type = server_conf.connection_type();

    let server_name = server_conf.server_name();
    let server_name = server_name.as_str();

    let port: u16 = server_conf.server_port();

    match connection_type {
        ConnectionType::Plain => {
            // run our app with hyper, listening globally on port 3001
            let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await ?;
            info!("Web server started on port [{port}]");
            axum::serve(listener, app_router).await ?;
        }

        ConnectionType::Ssl => {
            let rust_tls_config: RustlsConfig =
                if let (Some(SslConfValue::Path(server_key)), Some(SslConfValue::Path(server_cert))) =
                    (&server_conf.server_ssl_key(), &server_conf.server_ssl_cert()) {
                    RustlsConfig::from_pem_file(server_cert, server_key).await ?
                } else if let (Some(SslConfValue::Value(server_key)), Some(SslConfValue::Value(server_cert))) =
                    (&server_conf.server_ssl_key(), &server_conf.server_ssl_cert()) {
                    RustlsConfig::from_pem(
                        Vec::from(server_cert.as_bytes()),
                        Vec::from(server_key.as_bytes()),
                    ).await ?
                } else {
                    anyhow::bail!("Both {server_name} cert/key should have the same type")
                };

            use std::net::SocketAddr;
            let addr = SocketAddr::from(([0, 0, 0, 0], port));
            axum_server::bind_rustls(addr, rust_tls_config)
                .serve(app_router.into_make_service())
                .await ?;
        }

        ConnectionType::Auto =>
            anyhow::bail!("Server connection type auto detection is not supported"),
    }

    Ok(())
}
