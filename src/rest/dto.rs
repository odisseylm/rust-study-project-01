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


#[derive(PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[derive(educe::Educe)] #[educe(Debug)]
#[derive(derive_more::Display)]
#[display(fmt = "Amount {{ {} {} }}", value, currency)]
pub struct Amount {
    #[serde(with = "crate::json::serde_json_bd::bd_with")]
    #[educe(Debug(method(crate::entities::bd::bd_dbg_fmt)))]
    pub value: BigDecimal,
    // TODO: use simple validation
    pub currency: InnerCurStr, // , Currency  // Now it is String there just for projection's test
}



//--------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use crate::util::test_unwrap::TestSringOps;
    use crate::util::TestResultUnwrap;
    use super::*;

    #[test]
    fn display_test() {
        let amount = Amount {
            value: BigDecimal::from_str("123.0456").test_unwrap(),
            currency: InnerCurStr::const_make("USD"),
        };

        assert_eq!(amount.to_test_display_string(), "Amount { 123.0456 USD }");
        assert_eq!(
            amount.to_test_debug_string(),
            "Amount { value: 123.0456 (BigDecimal(sign=Plus, scale=4, digits=[1230456])), currency: USD }",
        );
    }
}
