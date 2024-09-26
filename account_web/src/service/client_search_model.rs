use log::error;
use mvv_common::{
    grpc::model::ConvertProtoEnumError,
    option_ext::DisplayOptionExt,
};
use crate::grpc_dependencies::mvv::client::search::api::v1::{
    Client as GrpcClientV1,
    PhoneNumber as GrpcPhoneNumberV1,
    client::ClientType as GrpcClientType,
    phone_number::PhoneType as GrpcPhoneType,
    client::Email,
};
//--------------------------------------------------------------------------------------------------



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
    pub client_type: ClientType,
    pub email: Option<String>,
}


#[derive(Debug)]
#[derive(strum_macros::Display)]
pub enum ClientType {
    #[strum(to_string = "General")]
    GeneralClient,
    #[strum(to_string = "Business")]
    BusinessClient,
    #[strum(to_string = "Super business")]
    SuperBusinessClient,
}
impl GrpcClientType {
    pub fn try_into_model(self) -> Result<ClientType, ConvertProtoEnumError> {
        match self {
            GrpcClientType::GeneralClient => Ok(ClientType::GeneralClient),
            GrpcClientType::BusinessClient => Ok(ClientType::BusinessClient),
            GrpcClientType::SuperBusinessClient => Ok(ClientType::SuperBusinessClient),
            // Or it should be error/warning error? Depends on requirements.
            GrpcClientType::Unspecified =>
                Err(ConvertProtoEnumError::unspecified_proto_enum_value(self as i32)),
        }
    }
}
impl TryFrom<GrpcClientType> for ClientType {
    type Error = ConvertProtoEnumError;
    fn try_from(value: GrpcClientType) -> Result<Self, Self::Error> {
        value.try_into_model()
    }
}

fn to_model_client_type(grpc_client_type_index: i32) -> Result<ClientType, ConvertProtoEnumError> {
    GrpcClientType::try_from(grpc_client_type_index)
        .map_err(ConvertProtoEnumError::from)
        .and_then(GrpcClientType::try_into_model)
}


#[derive(Debug)]
#[derive(strum_macros::Display)]
pub enum PhoneType {
    Mobile,
    Home,
    Work,
    Business,
}
/*
// No sense to use TryFrom
impl From<PhoneType> for GrpcPhoneType {
    fn from(value: PhoneType) -> Self {
        match value {
            PhoneType::Mobile => GrpcPhoneType::Mobile,
            PhoneType::Home => GrpcPhoneType::Home,
            PhoneType::Work => GrpcPhoneType::Work,
        }
    }
}
*/
impl GrpcPhoneType {
    pub fn try_into_model(self) -> Result<Option<PhoneType>, ConvertProtoEnumError> {
        match self {
            GrpcPhoneType::Mobile   => Ok(Some(PhoneType::Mobile)),
            GrpcPhoneType::Home     => Ok(Some(PhoneType::Home)),
            GrpcPhoneType::Work     => Ok(Some(PhoneType::Work)),
            GrpcPhoneType::Business => Ok(Some(PhoneType::Business)),
            // Or it should be error/warning error? Depends on requirements.
            GrpcPhoneType::Unspecified => Ok(None),
        }
    }
}
impl TryFrom<GrpcPhoneType> for Option<PhoneType> {
    type Error = ConvertProtoEnumError;
    fn try_from(value: GrpcPhoneType) -> Result<Self, Self::Error> {
        value.try_into_model()
    }
}


#[derive(Debug, derive_more::Display)]
#[display("{number} ({})", phone_type.as_only_some_to_display())]
pub struct PhoneNumber {
    pub number: String,
    pub phone_type: Option<PhoneType>,
}


fn to_model_phone_type_quietly(grpc_phone_type_index: i32) -> Option<PhoneType> {
    let phone_type_opt = GrpcPhoneType::try_from(grpc_phone_type_index)
        .map_err(ConvertProtoEnumError::from)
        .and_then(GrpcPhoneType::try_into_model)
        .unwrap_or_else(|err|{
            // I guess it is better to continue with such kind of error
            error!("{err:?}");
            None
        });
    phone_type_opt
}

impl PhoneNumber {
    fn from_grpc(phone_number: GrpcPhoneNumberV1) -> Option<PhoneNumber> {
        match phone_number.number {
            None => None,
            Some(number) => {
                let phone_type = to_model_phone_type_quietly(phone_number.r#type);
                Some(PhoneNumber { number, phone_type })
            }
        }
    }
}

impl GrpcClientV1 {
    pub fn try_into_model(self) -> anyhow::Result<ClientInfo> {
        let GrpcClientV1 { id, phones, first_name, last_name,
            birthday, active, client_type, email, ..} = self;
        let birthday = birthday.and_then(|birthday|
            chrono::NaiveDate::from_ymd_opt(birthday.year, birthday.month as u32, birthday.day as u32));
        let phones: Vec<PhoneNumber> = phones.into_iter()
            .filter_map(|p| PhoneNumber::from_grpc(p))
            .collect::<Vec<PhoneNumber>>();
        let email = email.map(|email|{
            match email { Email::EmailValue(email) => email }
        });
        let client_type = to_model_client_type(client_type) ?;

        Ok(ClientInfo {
            id, active,
            first_name, last_name,
            email, phones,
            birthday,
            client_type,
        })
    }
}

impl TryFrom<GrpcClientV1> for ClientInfo {
    type Error = anyhow::Error;
    fn try_from(value: GrpcClientV1) -> Result<Self, Self::Error> {
        value.try_into_model()
    }
}
