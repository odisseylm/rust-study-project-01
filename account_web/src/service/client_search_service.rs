use std::str::FromStr;
use std::sync::Arc;
use tonic::{Request, Status};
use mvv_common::grpc::{GrpcCallError, TonicErrToStatusExt};
use crate::grpc_dependencies::mvv::client::search::api::v1::{
    Client as GrpcClientV1,
    PhoneNumber as GrpcPhoneNumberV1,
    GetClientByIdRequest,
    client_search_service_client::ClientSearchServiceClient,
};
use tonic::transport::Channel;
use mvv_common::cfg::{load_optional_path_from_env_vars, load_url_from_env_var, SslConfValue};
use crate::grpc_dependencies::mvv::client::search::api::v1::client::Email;
//--------------------------------------------------------------------------------------------------



#[async_trait::async_trait]
pub trait ClientSearchService {
    async fn get_client_info(&self, client_id: &str) -> Result<Option<ClientInfo>, GrpcCallError>;
}


// Using grpc entities makes their usage in 'askama' much easier (due to Display support)
// and makes internal code less dependent on external services.
//
#[derive(Debug)]
pub struct ClientInfo {
    pub id: String,
    pub phones: Vec<PhoneNumber>,
    pub first_name: String,
    pub last_name: String,
    pub birthday: Option<chrono::NaiveDate>,
    pub active: bool,
    pub business_user: bool,
    pub super_business_user: bool,
    pub email: Option<String>,
}

#[derive(Debug, derive_more::Display)]
#[display("{number} ({phone_type})")]
pub struct PhoneNumber {
    pub number: String,
    pub phone_type: i32, // TODO: replace by enum with Display impl
}

impl PhoneNumber {
    fn from_grpc(phone_number: GrpcPhoneNumberV1) -> Option<PhoneNumber> {
        match phone_number.number {
            None => None,
            Some(number) =>
                Some(PhoneNumber { number, phone_type: phone_number.r#type })
        }
    }
}

impl From<GrpcClientV1> for ClientInfo {
    fn from(client: GrpcClientV1) -> Self {
        let GrpcClientV1 { id, phones, first_name, last_name,
            birthday, active, business_user, super_business_user, email, ..} = client;
        let birthday = birthday.and_then(|birthday|
            chrono::NaiveDate::from_ymd_opt(birthday.year, birthday.month as u32, birthday.day as u32));
        let phones: Vec<PhoneNumber> = phones.into_iter()
            .filter_map(|p| PhoneNumber::from_grpc(p))
            .collect::<Vec<PhoneNumber>>();
        let email = email.map(|email|{
            match email { Email::EmailValue(email) => email }
        });

        Self {
            id, first_name, last_name,
            active,
            email,
            phones,
            birthday,
            business_user, super_business_user,
        }
    }
}


pub struct ClientSearchServiceImpl {
    config: Arc<ClientSearchSoaCfg>,
    // client: crate::grpc_dependencies::mvv::client::search::api::v1::client_search_service_client::ClientSearchServiceClient<>,
}

pub struct GrpcClientAuthInterceptor {
    config: Arc<ClientSearchSoaCfg>,
}
impl tonic::service::Interceptor for GrpcClientAuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let cfg = &self.config;

        use axum_extra::headers::{ Authorization, authorization::Credentials };
        let auth = Authorization::basic(&cfg.user, &cfg.psw);

        let as_header_v = auth.0.encode();
        let as_header_v = as_header_v.to_str()
            .to_tonic_internal_err("GrpcClientAuthInterceptor. header name error") ?;

        request.metadata_mut().insert(
            "authorization",
            tonic::metadata::MetadataValue::from_str(as_header_v)
                .to_tonic_internal_err("GrpcClientAuthInterceptor. header value error") ?
        );

        Ok(request)
    }
}


#[async_trait::async_trait]
impl ClientSearchService for ClientSearchServiceImpl {

    async fn get_client_info(&self, client_id: &str) -> Result<Option<ClientInfo>, GrpcCallError> {

        let conf = self.config.clone();

        // TODO: can we use some pool ??
        let channel = Channel::from_shared(conf.base_url.clone())
            .map_err(|err|GrpcCallError::invalid_uri(conf.base_url.as_str(), err)) ?
            .connect().await ?;

        let mut client = ClientSearchServiceClient::with_interceptor(
            channel,
            GrpcClientAuthInterceptor { config: Arc::clone(&conf) },
        );

        let res = client.get_client_by_id(GetClientByIdRequest {
            client_id: client_id.to_owned(),
        }).await ?;

        let res = res.get_ref();
        let client = res.person.clone();

        Ok(client.map(|client|client.into()))
    }
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
pub struct ClientSearchSoaCfg {
    pub base_url: String,
    pub user: String,
    pub psw: String,
    pub server_cert: Option<SslConfValue>,
}

impl ClientSearchSoaCfg {

    pub fn load_from_env() -> anyhow::Result<Self> {
        let client_search_soa_cert = load_optional_path_from_env_vars([
            "DEPENDENCIES_CLIENT_SEARCH_SOA_SSL_CERT_PATH", "CLIENT_SEARCH_SOA_SSL_CERT_PATH"])
            ?.map(SslConfValue::Path);

        Ok(ClientSearchSoaCfg {
            // In general there may be several URLs with client balancing,
            // but now we use only 1st url
            base_url:
                load_url_from_env_var("DEPENDENCIES_CLIENT_SEARCH_SOA_GRPC_BASEURLS") ?,
            user:
                mvv_common::env::required_env_var("DEPENDENCIES_CLIENT_SEARCH_SOA_USER") ?,
            psw:
                mvv_common::env::required_env_var("DEPENDENCIES_CLIENT_SEARCH_SOA_PSW") ?,
            server_cert:
                client_search_soa_cert,
        })
    }
}



pub fn create_client_search_service(cfg: &ClientSearchSoaCfg)
    -> anyhow::Result<ClientSearchServiceImpl> {
    // TODO: use SSL

    Ok(ClientSearchServiceImpl { config: Arc::new(cfg.clone()) })

    /*
    let cert: Option<Certificate> = match cfg.server_cert {
        Some(SslConfValue::Path(ref cert_path)) =>{
            let pem = std::fs::read_to_string(cert_path)
                .map_err(|err| anyhow!("Error of reading from [{cert_path:?}] ({err:?})")) ?;
            Some(Certificate::from_pem(pem.as_bytes()) ?)
        }
        Some(SslConfValue::Value(ref value)) =>
            Some(Certificate::from_pem(value.as_bytes()) ?),
        None => None,
    };

    let mut client = reqwest::Client::builder()
        .default_headers({
            let mut headers = HeaderMap::new();
            // headers.insert("Authorization", HeaderValue::from_str(&basic_auth_creds.as_http_header()) ?);
            headers.insert("Authorization", auth.0.encode());
            headers
        });

    if let Some(cert) = cert {
        client = client.add_root_certificate(cert);
    }
    let client = client.build() ?;

    let client = AccountSoaRestClient::new_with_client(&cfg.base_url, client);
    let account_service = AccountServiceImpl { client };
    Ok(account_service)
    */
}



#[cfg(test)]
mod test {

    #[test]
    fn to_verify_compilation() {
    }

}