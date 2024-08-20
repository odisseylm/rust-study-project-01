use mvv_common::cfg::{load_path_from_env_var, SslConfValue};
//--------------------------------------------------------------------------------------------------


pub struct SslConfig {
    pub database_cert: SslConfValue,
    pub account_soa_key: SslConfValue,
    pub account_soa_cert: SslConfValue,
}

impl SslConfig {
    pub fn load_from_env() -> Result<Self, anyhow::Error> {
        Ok(Self {
            database_cert:
                SslConfValue::Path(load_path_from_env_var("DATABASE_SSL_CERT_PATH") ?),
            account_soa_key:
                SslConfValue::Path(load_path_from_env_var("ACCOUNT_SOA_SSL_KEY_PATH") ?),
            account_soa_cert:
                SslConfValue::Path(load_path_from_env_var("ACCOUNT_SOA_SSL_CERT_PATH") ?),
        })
    }
}
