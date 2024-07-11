use core::fmt;
use bigdecimal::BigDecimal;
use chrono::Utc;
use serde::{ Deserialize, Serialize };
use crate::entities::currency::InnerCurStr;
//--------------------------------------------------------------------------------------------------


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


// See https://crates.io/crates/axum-valid
#[derive(Debug, validator::Validate, serde::Deserialize)]
pub struct SomeRequest {
    #[validate(range(min = 1, max = 50))]
    pub page_size: usize,
    #[validate(range(min = 1))]
    pub page_no: usize,
}

/*
pub async fn pager_from_query(Valid(Query(pager)): Valid<Query<Pager>>) {
    assert!((1..=50).contains(&pager.page_size));
    assert!((1..).contains(&pager.page_no));
}

pub async fn pager_from_json(pager: Valid<Json<Pager>>) {
    assert!((1..=50).contains(&pager.page_size));
    assert!((1..).contains(&pager.page_no));
    // NOTE: all extractors provided support automatic dereferencing
    println!("page_no: {}, page_size: {}", pager.page_no, pager.page_size);
}
*/


#[derive(PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct Amount {
    #[serde(with = "crate::json::serde_json_bd::bd_with")]
    pub value: BigDecimal,
    // currency: Currency,
    // TODO: use simple validation
    pub currency: InnerCurStr, // Now it is String there just for projection's test
}
impl fmt::Debug for Amount { // TODO: why it manually written?
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Amount {{ {} {}  ({:?}) }}", self.value, self.currency, self.value)
    }
}
