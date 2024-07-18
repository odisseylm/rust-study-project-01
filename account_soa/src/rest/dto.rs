use bigdecimal::BigDecimal;
use chrono::Utc;
use serde::{ Deserialize, Serialize };
use once_cell::sync::Lazy;
use regex::Regex;
// use validator::Validate; // Need to do it manually since 'validator' does not import it by itself ?!
use mvv_common::entity::currency::InnerCurStr;
use mvv_common::unchecked::UncheckedResultUnwrap;
//--------------------------------------------------------------------------------------------------



static CURRENCY_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^[A-Z]{3}$"#).unchecked_unwrap()  // r"[a-z]{2}$"
});
static ID_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r#"^[0-9A-Za-z_\-]+$"#).unchecked_unwrap()  // r"[a-z]{2}$"
});


/*
// Need to have such function since 'validator' is not smart enough
// to use third-party strings.
#[inline]
fn cur_regex_validate(s: &str) -> Result<(), validator::ValidationError> {
    use crate::rest::valid::validator;
    validator::regex_validate(s, &CURRENCY_PATTERN)
}
*/



#[derive(PartialEq, Eq, Serialize, Deserialize)]
#[derive(educe::Educe)] #[educe(Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "{} {}", value, currency)]
// #[derive(validator::Validate)]
#[derive(validify::Validify)]
pub struct Amount {
    #[serde(with = "mvv_common::json::serde_json_bd::bd_with")]
    #[educe(Debug(method(mvv_common::entity::bd::bd_dbg_fmt)))]
    // #[validate(skip)] // for 'validator'
    pub value: BigDecimal,
    // 'validator' cannot automatically use third-party string, even if it has 'as_str()'...
    // #[validate(length(min=3, max=3), custom(function = cur_regex_validate))] // for 'validator'
    //
    // 'validify' cannot automatically use third-party strings for length validation, but it is ok with regex.
    #[validate(regex(CURRENCY_PATTERN))] // for 'validify'
    pub currency: InnerCurStr, // , Currency  // Now it is String there just for projection's test
}


#[derive(Debug, PartialEq)]
#[derive(Serialize, Deserialize)]
#[derive(derive_more::Display)]
#[display(fmt = "Account {{ {id}, iban: {iban}, client: {client_id}, amount: {amount}, created/updated at: {created_at}/{updated_at} }}")]
#[serde(rename_all = "camelCase")]
// #[derive(validator::Validate)]
#[derive(validify::Validify)]
pub struct Account {
    #[validate(length(min=1, max=320), regex(ID_PATTERN))]
    pub id: String, // TODO: use uuid
    #[validate(length(min=16, max=34), regex(ID_PATTERN))]
    pub iban: String,
    #[validate(length(min=1, max=320), regex(ID_PATTERN))]
    pub client_id: String, // TODO: use uuid
    #[validate(length(min=1, max=320))] // TODO: add simple validation // , regex(ID_PATTERN))]
    pub name: String,
    #[validify]
    pub amount: Amount,
    pub created_at: chrono::DateTime<Utc>,
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