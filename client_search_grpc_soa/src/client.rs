use chrono::Datelike;
use diesel::prelude::*;
use crate::grpc::{
    mvv::client::search::api::v1::{
        {Client, PhoneNumber},
        client::Email,
    },
    google::r#type::Date as ProtoDate,
};
//--------------------------------------------------------------------------------------------------



#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = crate::schema::CLIENTS)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ClientInfo {
    pub client_id: uuid::Uuid, // String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub birthday: chrono::NaiveDate,
    pub active: bool,
    pub business_user: bool,
    pub super_business_user: bool,
}

impl From<ClientInfo> for Client {
    fn from(value: ClientInfo) -> Self {
        Client {
            id: value.client_id.to_string(),
            email: Some(Email::EmailValue(value.email)),
            phones: vec!(PhoneNumber { number: Some(value.phone), r#type: 123 }),
            first_name: value.first_name,
            last_name: value.last_name,
            birthday: Some(ProtoDate {
                year: value.birthday.year(),
                month: value.birthday.month() as i32,
                day: value.birthday.day() as i32,
            }),
            active: value.active,
            business_user: value.business_user,
            super_business_user: value.super_business_user,
        }
    }
}
