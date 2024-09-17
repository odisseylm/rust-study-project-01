use tonic::transport::ServerTlsConfig;
use crate::cfg::{ServerConf, SslConfValue};
//--------------------------------------------------------------------------------------------------



pub async fn server_tls_config<Conf: ServerConf>(server_conf: &Conf)
    -> anyhow::Result<ServerTlsConfig> {

    let server_name = server_conf.server_name();
    let server_name = server_name.as_str();

    if let (Some(SslConfValue::Path(server_ssl_key)), Some(SslConfValue::Path(server_ssl_cert))) =
        (&server_conf.server_ssl_key(), &server_conf.server_ssl_cert()) {

        let server_ssl_key = std::fs::read_to_string(server_ssl_key) ?;
        let server_ssl_cert = std::fs::read_to_string(server_ssl_cert) ?;

        Ok(ServerTlsConfig::new()
            .identity(tonic::transport::Identity::from_pem(&server_ssl_cert, &server_ssl_key))
            // .client_auth_optional()
            // .client_ca_root()
        )

    } else if let (Some(SslConfValue::Value(server_ssl_key)), Some(SslConfValue::Value(server_ssl_cert))) =
        (&server_conf.server_ssl_key(), &server_conf.server_ssl_cert()) {

        Ok(ServerTlsConfig::new()
            .identity(tonic::transport::Identity::from_pem(
                &server_ssl_cert.as_bytes(), &server_ssl_key.as_bytes())))

    } else {
        anyhow::bail!("Both {server_name} cert/key should have the same type")
    }
}
