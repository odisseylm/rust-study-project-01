use mvv_common::{
    cfg::{load_optional_path_from_env_vars, BaseServerConf, ServerConf, SslConfValue},
    net::ConnectionType,
    string::StaticRefOrString,
};
use mvv_common::cfg::SslConfValueOptionExt;
//--------------------------------------------------------------------------------------------------


pub struct AccountSoaServerConfig {
    server_name: StaticRefOrString,
    server_env_name: StaticRefOrString,
    server_port: u16,
    connection_type: ConnectionType,
    server_ssl_key: Option<SslConfValue>,
    server_ssl_cert: Option<SslConfValue>,
    client_auth_ssl_ca_cert: Option<SslConfValue>,
    #[allow(dead_code)]
    database_cert: Option<SslConfValue>,
}

impl AccountSoaServerConfig {
    pub fn load_from_env(server_name: StaticRefOrString, server_env_name: StaticRefOrString)
        -> anyhow::Result<Self> where Self: Sized {

        let conf = BaseServerConf::load_from_env(server_name, server_env_name)?;
        let BaseServerConf {
            server_name, server_env_name, server_port,
            connection_type, server_ssl_key, server_ssl_cert,
            client_auth_ssl_ca_cert,
            ..
        } = conf;

        let database_cert = load_optional_path_from_env_vars([
            "POSTGRES_SSL_CERT_PATH", "DATABASE_SSL_CERT_PATH"])?.map(SslConfValue::Path);

        Ok(Self {
            server_name,
            server_env_name,
            server_port,
            connection_type,
            server_ssl_key,
            server_ssl_cert,
            database_cert,
            client_auth_ssl_ca_cert,
        })
    }
}

impl ServerConf for AccountSoaServerConfig {
    fn server_name(&self) -> &StaticRefOrString { &self.server_name }
    fn server_env_name(&self) -> &StaticRefOrString { &self.server_env_name }
    fn connection_type(&self) -> ConnectionType { self.connection_type }
    fn server_port(&self) -> u16 { self.server_port }
    fn server_ssl_key(&self) -> Option<&SslConfValue> { self.server_ssl_key.as_ref() }
    fn server_ssl_cert(&self) -> Option<&SslConfValue> { self.server_ssl_cert.as_ref() }
    fn client_auth_ssl_ca_cert(&self) -> Option<&SslConfValue> {
        self.client_auth_ssl_ca_cert.as_ref()
    }

    fn preload_values(self) -> anyhow::Result<Self> where Self: Sized {
        Ok(Self {
            server_name: self.server_name,
            server_env_name: self.server_env_name,
            server_port: self.server_port,
            connection_type: self.connection_type,
            server_ssl_key: self.server_ssl_key.preload() ?,
            server_ssl_cert: self.server_ssl_cert.preload() ?,
            client_auth_ssl_ca_cert: self.client_auth_ssl_ca_cert.preload() ?,
            database_cert: self.database_cert.preload() ?,
        })
    }
}
