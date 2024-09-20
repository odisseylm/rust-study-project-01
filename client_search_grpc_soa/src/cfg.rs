use mvv_common::{
    cfg::{BaseServerConf, ServerConf, SslConfValue},
    net::ConnectionType,
    string::StaticRefOrString,
};
//--------------------------------------------------------------------------------------------------


pub struct ClientSearchSoaServerConfig {
    pub base_server_conf: BaseServerConf,
}

impl ServerConf for ClientSearchSoaServerConfig {
    fn server_name(&self) -> &StaticRefOrString { self.base_server_conf.server_name() }
    fn server_env_name(&self) -> &StaticRefOrString { self.base_server_conf.server_env_name() }
    fn connection_type(&self) -> ConnectionType { self.base_server_conf.connection_type() }
    fn server_port(&self) -> u16 { self.base_server_conf.server_port() }
    fn server_ssl_key(&self) -> Option<&SslConfValue> { self.base_server_conf.server_ssl_key() }
    fn server_ssl_cert(&self) -> Option<&SslConfValue> { self.base_server_conf.server_ssl_cert() }
    fn client_auth_ssl_ca_cert(&self) -> Option<&SslConfValue> {
        self.base_server_conf.client_auth_ssl_ca_cert()
    }

    fn preload_values(self) -> anyhow::Result<Self> where Self: Sized {
        Ok(Self {
            base_server_conf: self.base_server_conf.preload_values() ?,
        })
    }
}

impl ClientSearchSoaServerConfig {
    pub fn load_from_env() -> anyhow::Result<Self> where Self: Sized {
        let base_server_conf = BaseServerConf::load_from_env(
            "client_search_soa".into(), "CLIENT_SEARCH_SOA".into()) ?;
        Ok(ClientSearchSoaServerConfig { base_server_conf })
    }
}
