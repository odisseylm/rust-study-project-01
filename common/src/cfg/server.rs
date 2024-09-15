use std::time::Duration;
use crate::cfg::SslConfValue;
use crate::net::ConnectionType;
use crate::string::StaticRefOrString;
use super::{get_server_port, get_server_connection_type, load_path_from_env_vars};
//--------------------------------------------------------------------------------------------------



pub trait ServerConf {
    fn server_name(&self) -> &StaticRefOrString;
    /// Should be uppercase.
    fn server_env_name(&self) -> &StaticRefOrString;
    fn connection_type(&self) -> ConnectionType;
    /// Main server port.
    fn server_port(&self) -> u16;
    fn server_ssl_key(&self) -> Option<&SslConfValue>;
    fn server_ssl_cert(&self) -> Option<&SslConfValue>;
    fn shutdown_timeout(&self) -> Duration {
        Duration::from_millis(5)
    }
}


pub struct BaseServerConf {
    pub server_name: StaticRefOrString,
    pub server_env_name: StaticRefOrString,
    pub server_port: u16,
    pub connection_type: ConnectionType,
    pub server_ssl_key: Option<SslConfValue>,
    pub server_ssl_cert: Option<SslConfValue>,
}

impl BaseServerConf {
    pub fn load_from_env(server_name: StaticRefOrString, server_env_name: StaticRefOrString)
                         -> anyhow::Result<Self> where Self: Sized {

        let server_name = server_name.clone();
        let server_env_name = server_env_name.clone();
        let server_port = get_server_port(server_env_name.as_str())?;
        let connection_type = get_server_connection_type(server_env_name.as_str())?;

        let server_ssl_key: Option<SslConfValue>;
        let server_ssl_cert: Option<SslConfValue>;

        if let ConnectionType::Ssl = connection_type {
            server_ssl_key = Some(SslConfValue::Path(load_path_from_env_vars([
                // for local dev testing with single config env file
                &format!("{server_env_name}SSL_KEY_PATH"), &format!("{server_env_name}_SSL_KEY_PATH"),
                // for prod/docker
                "SERVER_SSL_KEY_PATH", "SSL_KEY_PATH"])?));

            server_ssl_cert = Some(SslConfValue::Path(load_path_from_env_vars([
                // for local dev testing with single config env file
                &format!("{server_env_name}SSL_CERT_PATH"), &format!("{server_env_name}_SSL_CERT_PATH"),
                // for prod/docker
                "SERVER_SSL_CERT_PATH", "SSL_CERT_PATH"])?));
        } else {
            server_ssl_key = None;
            server_ssl_cert = None;
        }

        Ok(Self {
            server_name,
            server_env_name,
            server_port,
            connection_type,
            server_ssl_key,
            server_ssl_cert,
        })
    }
}

impl ServerConf for BaseServerConf {
    fn server_name(&self) -> &StaticRefOrString { &self.server_name }
    fn server_env_name(&self) -> &StaticRefOrString { &self.server_env_name }
    fn connection_type(&self) -> ConnectionType { self.connection_type }
    fn server_port(&self) -> u16 { self.server_port }
    fn server_ssl_key(&self) -> Option<&SslConfValue> { self.server_ssl_key.as_ref() }
    fn server_ssl_cert(&self) -> Option<&SslConfValue> { self.server_ssl_cert.as_ref() }
}
