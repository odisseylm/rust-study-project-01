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
        value.into_grpc()
    }
}

impl ClientInfo {
    pub fn into_grpc(self) -> Client {
        Client {
            id: self.client_id.to_string(),
            email: Some(Email::EmailValue(self.email)),
            phones: vec!(PhoneNumber { number: Some(self.phone), r#type: 123 }),
            first_name: self.first_name,
            last_name: self.last_name,
            birthday: Some(ProtoDate {
                year: self.birthday.year(),
                month: self.birthday.month() as i32,
                day: self.birthday.day() as i32,
            }),
            active: self.active,
            business_user: self.business_user,
            super_business_user: self.super_business_user,
        }
    }
}
