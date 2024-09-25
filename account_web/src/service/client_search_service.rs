use std::borrow::Cow;
use std::sync::Arc;
use log::info;
use crate::grpc_dependencies::mvv::client::search::api::v1::{
    Client as GrpcClientV1,
    PhoneNumber as GrpcPhoneNumberV1,
    GetClientByIdRequest,
    client_search_service_client::ClientSearchServiceClient,
};
use tonic::transport::{Channel, ClientTlsConfig};
use mvv_auth::grpc::client::GrpcClientAuthInterceptor;
use mvv_common::{
    grpc::GrpcCallError,
    cfg::{BaseDependencyConnectConf, DependencyConnectConf},
    secure::SecureString,
};
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
            id, active,
            first_name, last_name,
            email, phones,
            birthday,
            business_user, super_business_user,
        }
    }
}


pub struct ClientSearchServiceImpl {
    config: Arc<BaseDependencyConnectConf>,
}


#[async_trait::async_trait]
impl ClientSearchService for ClientSearchServiceImpl {

    async fn get_client_info(&self, client_id: &str) -> Result<Option<ClientInfo>, GrpcCallError> {

        let conf = self.config.clone();

        let certs = get_ca_and_server_certs(conf.as_ref()) ?;

        // TODO: can we use some pool ??
        let channel = Channel::from_shared(conf.base_url().to_string())
            .map_err(|err|GrpcCallError::invalid_uri(conf.base_url().as_str(), err)) ?;

        let channel =
            if !certs.is_empty() {
                let tls = ClientTlsConfig::new()
                    // It can be added to DependencyConnectConf.
                    // 'domain_name' may be needed if we connect by IP.
                    //
                    // .domain_name("example.com")
                    .ca_certificates(certs.into_iter())
                    ;

                channel.tls_config(tls) ?
            } else {
                channel
            };

        let channel = channel.connect().await ?;

        let mut client = ClientSearchServiceClient::with_interceptor(
            channel, GrpcClientAuthInterceptor { config: Arc::clone(&conf) },
        );

        let res = client.get_client_by_id(GetClientByIdRequest {
            client_id: client_id.to_owned(),
        }).await ?;

        let res = res.get_ref();
        let client = res.client.clone();

        Ok(client.map(|client|client.into()))
    }
}


fn get_ca_and_server_certs<Conf: DependencyConnectConf>(conf: &Conf)
    -> anyhow::Result<Vec<tonic::transport::Certificate>> {

    let certs = [conf.ca_cert(), conf.server_cert()]
        .into_iter()
        .filter_map(|opt|opt.as_ref())
        .map(|cert|cert.as_secure_string())
        .collect::<anyhow::Result<Vec<Cow<SecureString>>>>() ?;

    let certs = certs.into_iter()
        .map(|cert|tonic::transport::Certificate::from_pem(cert.as_ref().as_bytes()))
        .collect::<Vec<_>>();
    Ok(certs)
}


pub fn create_client_search_service(cfg: &BaseDependencyConnectConf)
    -> anyhow::Result<ClientSearchServiceImpl> {

    info!("Creating client-search service base on config [{cfg:?}]");
    Ok(ClientSearchServiceImpl { config: Arc::new(cfg.clone()) })
}



#[cfg(test)]
mod test {
    #[test]
    fn to_verify_compilation() {
    }
}
