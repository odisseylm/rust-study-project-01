use std::str::FromStr;
use bigdecimal::BigDecimal;
use project01::entities::currency::InnerCurStr;
use project01::rest::dto::Amount;
use project01::util::test_unwrap::TestSringOps;
use project01::util::TestResultUnwrap;
//--------------------------------------------------------------------------------------------------



#[test]
fn display_and_debug_fmt_test() {
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
