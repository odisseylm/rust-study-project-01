use anyhow::anyhow;
use reqwest::Certificate;
use crate::cfg::SslConfValue;
use crate::secure::SecureString;
use crate::string::StaticRefOrString;
use super::{load_url_from_env_vars, required_env_var, load_optional_path_from_env_vars};

#[derive(Debug, Copy, Clone)]
pub enum DependencyType {
    REST,
    GRPC,
}
impl DependencyType {
    pub fn as_env_part(&self) -> &'static str {
        match self {
            DependencyType::REST => "REST",
            DependencyType::GRPC => "GRPC",
        }
    }
}

static EMPTY_STATIC_REF_OR_STRING: StaticRefOrString = StaticRefOrString::Ref("");

pub trait DependencyConnectConf: core::fmt::Debug {
    /// mainly for trace (maybe required for some SOAs)
    fn app_name(&self) -> &StaticRefOrString;
    /// Should be uppercase.
    fn dep_env_name(&self) -> &StaticRefOrString;
    fn dependency_type(&self) -> Option<DependencyType>;
    fn base_url(&self) -> &StaticRefOrString {
        self.base_urls().first().unwrap_or_else(||&EMPTY_STATIC_REF_OR_STRING)
    }
    fn base_urls(&self) -> &Vec<StaticRefOrString>;
    fn user(&self) -> &Option<StaticRefOrString>;
    fn password(&self) -> &Option<SecureString>;
    fn server_cert(&self) -> &Option<SslConfValue>;
    fn client_cert(&self) -> &Option<SslConfValue>;
}



// - DEPENDENCIES_ACCOUNTSOA_REST_BASEURLS=https://account-soa/account-soa/api
// - DEPENDENCIES_ACCOUNTSOA_REST_BASEURLTEMPLATE=https://bank-plugin-account-soa-REPLICA_NUMBER/account-soa/api
// - DEPENDENCIES_ACCOUNTSOA_REST_CONTEXTPATH=/account-soa/api
// - DEPENDENCIES_ACCOUNTSOA_REST_REPLICACOUNT=${DOCKER_COMPOSE_SCALE_REPLICA_COUNT}
// - DEPENDENCIES_ACCOUNTSOA_REST_REPLICAIDTYPE=OneBasedIncremented
// - DEPENDENCIES_ACCOUNTSOA_REST_PINGTIMEOUT=5s
// - DEPENDENCIES_ACCOUNTSOA_REST_BALANCERTYPE=ribbon
//

#[derive(Debug, Clone)]
pub struct BaseDependencyConnectConf {
    pub app_name: StaticRefOrString,

    pub dep_env_name: StaticRefOrString,
    pub dep_type: DependencyType,
    // In general there may be several URLs with client balancing,
    // but now we use only 1st url
    pub base_urls: Vec<StaticRefOrString>,
    pub user: Option<StaticRefOrString>,
    pub password: Option<SecureString>,
    pub server_cert: Option<SslConfValue>,
    pub client_cert: Option<SslConfValue>,
}

impl BaseDependencyConnectConf {
    pub fn load_from_env(dependency_env_name: StaticRefOrString, dep_type: DependencyType,
                         app_name: StaticRefOrString)
                         -> anyhow::Result<Self> where Self: Sized {

        let dep_type_env_name = dep_type.as_env_part();
        let base_url = load_url_from_env_vars([
            &format!("DEPENDENCIES_{dependency_env_name}_{dep_type_env_name}_BASE_URLS"),
            &format!("DEPENDENCIES_{dependency_env_name}_{dep_type_env_name}_BASEURLS"),
            &format!("DEPENDENCIES_{dependency_env_name}_BASE_URLS"),
            &format!("DEPENDENCIES_{dependency_env_name}_BASEURLS"),
        ]) ?;

        let user = required_env_var(&format!("DEPENDENCIES_{dependency_env_name}_USER")) ?;
        let psw = required_env_var(&format!("DEPENDENCIES_{dependency_env_name}_PSW")) ?;

        let server_cert = load_optional_path_from_env_vars([
                &format!("DEPENDENCIES_{dependency_env_name}_SERVER_SSL_CERT_PATH"),
                &format!("DEPENDENCIES_{dependency_env_name}_SSL_CERT_PATH"),
                &format!("{dependency_env_name}_SERVER_SSL_CERT_PATH"),
                &format!("{dependency_env_name}_SSL_CERT_PATH"),
            ])
            ?.map(SslConfValue::Path);

        let client_cert = load_optional_path_from_env_vars([
                &format!("DEPENDENCIES_{dependency_env_name}_CLIENT_SSL_CERT_PATH"),
                &format!("{dependency_env_name}_CLIENT_SSL_CERT_PATH"),
            ])
            ?.map(SslConfValue::Path);

        Ok(BaseDependencyConnectConf {
            app_name,
            dep_env_name: dependency_env_name.into(),
            dep_type,
            // now only ONE url is used
            base_urls: vec!(base_url.into()),
            user: Some(user.into()),
            password: Some(psw.into()),
            server_cert,
            client_cert,
        })
    }
}

impl DependencyConnectConf for BaseDependencyConnectConf {
    fn app_name(&self) -> &StaticRefOrString {
        &self.app_name
    }
    fn dep_env_name(&self) -> &StaticRefOrString {
        &self.dep_env_name
    }
    fn dependency_type(&self) -> Option<DependencyType> {
        Some(self.dep_type)
    }
    fn base_urls(&self) -> &Vec<StaticRefOrString> {
        &self.base_urls
    }
    fn user(&self) -> &Option<StaticRefOrString> {
        &self.user
    }
    fn password(&self) -> &Option<SecureString> {
        &self.password
    }
    fn server_cert(&self) -> &Option<SslConfValue> {
        &self.server_cert
    }
    fn client_cert(&self) -> &Option<SslConfValue> {
        &self.client_cert
    }
}


pub fn to_reqwest_tls_cert(cert: &Option<SslConfValue>) -> anyhow::Result<Option<Certificate>> {
    let cert = match cert {
        Some(SslConfValue::Path(ref cert_path)) =>{
            let pem = std::fs::read_to_string(cert_path)
                .map_err(|err| anyhow!("Error of reading from [{cert_path:?}] ({err:?})")) ?;
            Some(Certificate::from_pem(pem.as_bytes()) ?)
        }
        Some(SslConfValue::Value(ref value)) =>
            Some(Certificate::from_pem(value.as_bytes()) ?),
        None => None,
    };
    Ok(cert)
}
