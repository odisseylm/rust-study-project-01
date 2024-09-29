use std::io::Write;
use chrono::Datelike;
use mvv_common::{
    generate_diesel2_pg_from_sql_from_str, generate_diesel2_pg_to_sql_from_as_str,
    db::DbMappingError,
};
use crate::grpc::{
    mvv::client::search::api::v1::{
        {Client as GrpcClient, PhoneNumber},
        client::{Email, ClientType as GrpcClientType},
        phone_number::PhoneType as GrpcPhoneType,
    },
    google::r#type::Date as ProtoDate,
};
//--------------------------------------------------------------------------------------------------


#[derive(Debug, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = crate::schema::CLIENTS)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct ClientInfo {
    pub client_id: uuid::Uuid, // String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: String,
    pub phone_type: PhoneType,
    pub birthday: chrono::NaiveDate,
    pub active: bool,
    pub business_user: bool,
    pub super_business_user: bool,
}

impl TryFrom<ClientInfo> for GrpcClient {
    type Error = anyhow::Error;
    fn try_from(value: ClientInfo) -> Result<Self, Self::Error> {
        value.try_into_grpc()
    }
}


impl ClientInfo {
    pub fn try_into_grpc(self) -> anyhow::Result<GrpcClient> {
        let client_type =
            if self.super_business_user {
                GrpcClientType::SuperBusinessClient
            } else if self.business_user {
                GrpcClientType::BusinessClient
            } else {
                GrpcClientType::GeneralClient
            };

        Ok(GrpcClient {
            id: self.client_id.to_string(),
            email: Some(Email::EmailValue(self.email)),
            phones: vec!(PhoneNumber {
                number: Some(self.phone),
                r#type: self.phone_type.try_into_grpc() ? as i32,
            }),
            first_name: self.first_name,
            last_name: self.last_name,
            birthday: Some(ProtoDate {
                year: self.birthday.year(),
                month: self.birthday.month() as i32,
                day: self.birthday.day() as i32,
            }),
            active: self.active,
            client_type: client_type as i32,
        })
    }
}


#[derive(Debug)]
#[derive(strum_macros::Display)]
#[derive(diesel::AsExpression, diesel::FromSqlRow)]
#[diesel(sql_type = diesel::sql_types::VarChar)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub enum PhoneType {
    Mobile,
    Home,
    Work,
    Business,
}
impl PhoneType {
    fn as_db_str(&self) -> &'static str {
        match self {
            PhoneType::Mobile => "M",
            PhoneType::Home => "H",
            PhoneType::Work => "W",
            PhoneType::Business => "B",
        }
    }
    // Using fixedstr::str8 instead of standard String it is just experiment/investigation,
    // like tiny optimization to avoid using heap if it can be skipped.
    fn try_from_db_str(db_str: &str) -> Result<Self, DbMappingError<fixedstr::str32>> {
        match db_str {
            "M" => Ok(PhoneType::Mobile),
            "H" => Ok(PhoneType::Home),
            "W" => Ok(PhoneType::Work),
            "B" => Ok(PhoneType::Business),
            _other =>
                Err(DbMappingError::unexpect_db_tiny_str(db_str, "CLIENTS", "PHONE_TYPE")),
        }
    }
}
impl PhoneType {
    pub fn try_into_grpc(self) -> anyhow::Result<GrpcPhoneType> {
        match self {
            PhoneType::Mobile => Ok(GrpcPhoneType::Mobile),
            PhoneType::Home => Ok(GrpcPhoneType::Home),
            PhoneType::Work => Ok(GrpcPhoneType::Work),
            PhoneType::Business => Ok(GrpcPhoneType::Business),
        }
    }
}
impl TryFrom<PhoneType> for GrpcPhoneType {
    type Error = anyhow::Error;
    fn try_from(value: PhoneType) -> Result<Self, Self::Error> {
        value.try_into_grpc()
    }
}
// generate_pg_delegate_type_info!{ PhoneType, str }
// generate_pg_encode_from_as_str!{ PhoneType, as_db_str }
// generate_pg_decode_from_str!{ PhoneType, try_from_db_str }


/*
impl<B: Backend<BindCollector<'_> = RawBytesBindCollector<B>>> serialize::ToSql<VarChar, B> for PhoneType
    where String: serialize::ToSql<VarChar, B> {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, B>) -> serialize::Result {
        <str as serialize::ToSql<Char, B>>::to_sql(self.as_db_str(), out)
    }
}

impl<B: Backend> deserialize::FromSql<VarChar, B> for PhoneType
    where String: deserialize::FromSql<VarChar, B> {
    fn from_sql(bytes: RawValue<B>) -> deserialize::Result<Self> {
        let aa: PhoneType = <String as deserialize::FromSql<VarChar, B>>::from_sql(bytes)
            .map(|db_str| PhoneType::try_from_db_str(db_str.as_str())
                //.map_err()
            ) ?;
        Ok(aa)
    }
}
*/
/*
fn aa() -> diesel::deserialize::Result<()> {
    let bytes = "".as_bytes();
    let report_bytes_len = min(bytes.len(), 10);
    let _aa = core::str::from_utf8(bytes)
        .map_err(|_utf8err| DbMappingError::IncorrectUt8DbValue {
            // value: &bytes[0..report_bytes_len].clone(),
            value: Vec::from(&bytes[0..report_bytes_len]),
            table: "dsd".into(),
            column: "sd".into(),
            backtrace: backtrace(),
        }) ?;

    Ok(())
}
*/

// generate_diesel2_pg_from_sql_from_str_lossy! { PhoneType, try_from_db_str }
generate_diesel2_pg_from_sql_from_str! { PhoneType, try_from_db_str, "CLIENTS", "PHONE_TYPE" }
generate_diesel2_pg_to_sql_from_as_str! { PhoneType, as_db_str }


/*
impl diesel::serialize::ToSql<diesel::sql_types::VarChar, diesel::pg::Pg> for PhoneType {
    fn to_sql<'b>(&'b self, out: &mut diesel::serialize::Output<'b, '_, diesel::pg::Pg>)
        -> diesel::serialize::Result {
        let as_db_str = self.as_db_str();
        out.write_all(as_db_str.as_bytes()) ?;
        Ok(diesel::serialize::IsNull::No)
    }
}

impl diesel::deserialize::FromSql<diesel::sql_types::VarChar, diesel::pg::Pg> for PhoneType {
    fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
        let str = String::from_utf8_lossy(bytes.as_bytes());
        let str = str.as_ref();
        let value = PhoneType::try_from_db_str(str) ?;
        Ok(value)
    }
}
*/


#[cfg(test)]
mod tests {
    #[test]
    fn verify_compilation_success() {
    }
}