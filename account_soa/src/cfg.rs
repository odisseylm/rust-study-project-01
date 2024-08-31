use mvv_common::{
    cfg::{load_optional_path_from_env_vars, BaseServerConf, ServerConf, SslConfValue},
    net::ConnectionType,
    string::StaticRefOrString,
};
//--------------------------------------------------------------------------------------------------


pub struct AccountSoaServerConfig {
    server_name: StaticRefOrString,
    server_env_name: StaticRefOrString,
    server_port: u16,
    connection_type: ConnectionType,
    server_ssl_key: Option<SslConfValue>,
    server_ssl_cert: Option<SslConfValue>,
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
}
