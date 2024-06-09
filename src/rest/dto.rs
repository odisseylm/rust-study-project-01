use bigdecimal::BigDecimal;
use chrono::Utc;
use serde::{ Deserialize, Serialize };


// pub type Amount = crate::entities::amount::Amount;
// pub type Id = crate::entities::id::Id;


#[derive(Debug)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub id: String,
    pub user_id: String,
    pub amount: Amount,
    pub created_at: chrono::DateTime<Utc>,
    // #[serde(serialize_with = "serialize_fn...")]
    pub updated_at: chrono::DateTime<Utc>,
}


#[derive(PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)] // TODO: move it to DTO
pub struct Amount {
    #[serde(with = "crate::entities::serde_json_bd::bd_with")]
    pub value: BigDecimal,
    // currency: Currency,
    pub currency: String, // Now it is String there just for projection's test
}
impl core::fmt::Debug for Amount {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Amount {{ {} {}  ({:?}) }}", self.value, self.currency, self.value)
    }
}
