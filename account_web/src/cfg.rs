use mvv_common::cfg::{load_path_from_env_vars, SslConfValue};
//--------------------------------------------------------------------------------------------------


pub struct SslConfig {
    pub server_ssl_key: SslConfValue,
    pub server_ssl_cert: SslConfValue,

    // pub database_cert: SslConfValue,
    pub account_soa_cert: SslConfValue,
}

impl SslConfig {
    pub fn load_from_env() -> Result<Self, anyhow::Error> {
        Ok(Self {
            server_ssl_key:
                SslConfValue::Path(load_path_from_env_vars([
                    // for local dev testing with single config env file
                    "ACCOUNT_WEB_SSL_KEY_PATH",
                    // for prod/docker
                    "SERVER_SSL_KEY_PATH", "SSL_KEY_PATH"]) ?),
            server_ssl_cert:
                SslConfValue::Path(load_path_from_env_vars([
                    // for local dev testing with single config env file
                    "ACCOUNT_WEB_SSL_CERT_PATH",
                    // for prod/docker
                    "SERVER_SSL_CERT_PATH", "SSL_CERT_PATH"]) ?),

            // database_cert:
            //    SslConfValue::Path(load_path_from_env_vars(["POSTGRES_SSL_CERT_PATH", "DATABASE_SSL_CERT_PATH"]) ?),
            account_soa_cert:
            SslConfValue::Path(load_path_from_env_vars(["DEPENDENCIES_ACCOUNT_SOA_SSL_CERT_PATH", "ACCOUNT_SOA_SSL_CERT_PATH"]) ?),
        })
    }
}
