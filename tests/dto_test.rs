use core::str::FromStr;
use indoc::indoc;
use bigdecimal::BigDecimal;
use chrono::{ Utc, FixedOffset };
use project01::entities::currency::InnerCurStr;
use project01::rest::dto::{Account, Amount};
use project01::util::test_unwrap::TestSringOps;
use project01::util::{TestOptionUnwrap, TestResultUnwrap};
//--------------------------------------------------------------------------------------------------



#[test]
fn amount_display_and_debug_fmt_test() {
    let amount = Amount {
        value: BigDecimal::from_str("123.0456").test_unwrap(),
        currency: InnerCurStr::const_make("USD"),
    };

    assert_eq!(amount.to_test_display_string(), "123.0456 USD");
    assert_eq!(
            amount.to_test_debug_string(),
            "Amount { value: 123.0456 (BigDecimal(sign=Plus, scale=4, digits=[1230456])), currency: USD }",
        );
}


#[test]
fn amount_to_json_test() {
    let amount = Amount {
        value: BigDecimal::from_str("123.0456").test_unwrap(),
        currency: InnerCurStr::const_make("USD"),
    };
    assert_eq!(serde_json::to_string(&amount).test_unwrap(), r#"{"value":123.0456,"currency":"USD"}"#);

    let amount = Amount {
        value: BigDecimal::from_str("123.0456000000000000000000000000789").test_unwrap(),
        currency: InnerCurStr::const_make("USD"),
    };
    assert_eq!(
        serde_json::to_string(&amount).test_unwrap(),
        r#"{"value":123.0456000000000000000000000000789,"currency":"USD"}"#,
    );
}


#[test]
fn amount_from_json_test() {
    let amount_from_json: Amount = serde_json::from_str(r#"{"value":123.0456,"currency":"USD"}"#)
        .test_unwrap();
    assert_eq!(
        amount_from_json,
        Amount {
            value: BigDecimal::from_str("123.0456").test_unwrap(),
            currency: InnerCurStr::const_make("USD"),
        },
    );

    let amount_from_json: Amount = serde_json::from_str(
        r#"{"value":123.0456000000000000000000000000789,"currency":"USD"}"#).test_unwrap();
    assert_eq!(
        amount_from_json,
        Amount {
            value: BigDecimal::from_str("123.0456000000000000000000000000789").test_unwrap(),
            currency: InnerCurStr::const_make("USD"),
        },
    );
}


#[test]
fn account_display_and_debug_fmt_test() {
    let account = Account {
        id: "abcdef-123".to_test_string(),
        user_id: "qwerty-456".to_test_string(),
        amount: Amount {
            value: BigDecimal::from_str("123.0456").test_unwrap(),
            currency: InnerCurStr::const_make("USD"),
        },
        created_at: chrono::DateTime::<FixedOffset>::from_str("2024-05-30 22:29:57 +02:00")
            .test_unwrap().to_utc(),
        updated_at: chrono::DateTime::<FixedOffset>::from_str("2024-05-31 22:29:57 +02:00")
            .test_unwrap().to_utc(),
    };

    assert_eq!(
        account.to_test_display_string(),
        "Account { abcdef-123, user: qwerty-456, amount: 123.0456 USD, created/updated at: 2024-05-30 20:29:57 UTC/2024-05-31 20:29:57 UTC }",
    );
    assert_eq!(
            account.to_test_debug_string(),
            "Account { id: \"abcdef-123\", user_id: \"qwerty-456\", \
             amount: Amount { value: 123.0456 (BigDecimal(sign=Plus, scale=4, digits=[1230456])), currency: USD }, \
             created_at: 2024-05-30T20:29:57Z, updated_at: 2024-05-31T20:29:57Z }",
        );
}


#[test]
fn account_to_json() {
    let account = Account {
        id: "abcdef-123".to_test_string(),
        user_id: "qwerty-456".to_test_string(),
        amount: Amount {
            value: BigDecimal::from_str("123.0456").test_unwrap(),
            currency: InnerCurStr::const_make("USD"),
        },
        created_at: chrono::DateTime::<FixedOffset>::from_str("2024-05-30 22:29:57 +02:00")
            .test_unwrap().to_utc(),
        updated_at: chrono::DateTime::<FixedOffset>::from_str("2024-05-31 22:29:57 +02:00")
            .test_unwrap().to_utc(),
    };

    let account_json = serde_json::to_string(&account).test_unwrap();
    assert_eq!(account_json, const_str::replace!( indoc! {r#"
               {"id":"abcdef-123",
               "userId":"qwerty-456",
               "amount":{"value":123.0456,"currency":"USD"},
               "createdAt":"2024-05-30T20:29:57Z",
               "updatedAt":"2024-05-31T20:29:57Z"}
               "#}, "\n", "")
    );

    assert_json_diff::assert_json_eq!(
        serde_json::Value::from_str(account_json.as_str()).test_unwrap(),
        serde_json::json!(
            {
            "id": "abcdef-123",
            "userId": "qwerty-456",
            "amount": { "value": 123.0456, "currency":"USD" },
            // "amount": "123.0456 USD",
            "createdAt": "2024-05-30T20:29:57Z",
            "updatedAt": "2024-05-31T20:29:57Z",
            }
        )
    );
}


#[test]
fn account_from_json() {

    let as_json = serde_json::json!(
            {
            "id": "abcdef-123",
            "userId": "qwerty-456",
            "amount": { "value": 123.0456, "currency":"USD" },
            // "amount": "123.0456 USD",
            "createdAt": "2024-05-30T20:29:57Z",
            "updatedAt": "2024-05-31T20:29:57Z",
            }
        );

    let account_from_json: Account = serde_json::from_str(&as_json.to_test_string()).test_unwrap();

    assert_eq!(account_from_json.id, "abcdef-123");
    assert_eq!(account_from_json.user_id, "qwerty-456");
    assert_eq!(account_from_json.amount, Amount {
        value: BigDecimal::from_str("123.0456").test_unwrap(),
        currency: InnerCurStr::from_str("USD").test_unwrap(),
    });
    assert_eq!(account_from_json.created_at, chrono::DateTime::<Utc>::from_str("2024-05-30T20:29:57Z").test_unwrap());
    assert_eq!(account_from_json.updated_at, chrono::DateTime::<Utc>::from_str("2024-05-31T20:29:57Z").test_unwrap());


    let orig_account = Account {
        id: "abcdef-123".to_test_string(),
        user_id: "qwerty-456".to_test_string(),
        amount: Amount {
            value: BigDecimal::from_str("123.0456").test_unwrap(),
            currency: InnerCurStr::const_make("USD"),
        },
        created_at: chrono::DateTime::<FixedOffset>::from_str("2024-05-30 22:29:57 +02:00")
            .test_unwrap().to_utc(),
        updated_at: chrono::DateTime::<FixedOffset>::from_str("2024-05-31 22:29:57 +02:00")
            .test_unwrap().to_utc(),
    };

    assert_eq!(account_from_json, orig_account);
}


#[test]
fn validate_account_test() {
    let as_json = serde_json::json!(
            {
            "id": "abcdef-123",
            "userId": "qwerty-456",
            "amount": { "value": 123.0456, "currency":"us2" },// "ДОЛ" },
            // "amount": "123.0456 USD",
            "createdAt": "2024-05-30T20:29:57Z",
            "updatedAt": "2024-05-31T20:29:57Z",
            }
        );

    let account_dirty_obj: Account = serde_json::from_str(&as_json.to_test_string()).test_unwrap();

    use validator::Validate;
    let valid_res = account_dirty_obj.validate();
    assert_eq!(
        valid_res.clone().err().test_unwrap().to_test_display_string(),
        r#"amount.currency: Validation error: regex [{"value": String("us2")}]"#,
    );
    assert_eq!(
        valid_res.err().test_unwrap().to_test_debug_string(),
        r#"ValidationErrors({"amount": Struct(ValidationErrors({"currency": Field([ValidationError { code: "regex", message: None, params: {"value": String("us2")} }])}))})"#,
    );
}


/*
#[test]
fn main_test()  {
    let string1 = "username";
    let string2 = "password";

    // let sasl_jaas_config = format!( const_str::replace! { indoc::indoc! {
    let sasl_jaas_config = format!( indoc::indoc! {
        r#"
        org.apache.kafka.common.security.plain.PlainLoginModule #
        required username="{}"
        password="{}";
        "#},
        string1,
        string2);

    println!("{}", sasl_jaas_config);
    println!("done");
    assert_eq!(1, 2);
}
*/
