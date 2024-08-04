use chrono::prelude::*;
use type_rules::prelude::*;
use mvv_common::test::{ TestDisplayStringOps, TestResultUnwrap };
//--------------------------------------------------------------------------------------------------


#[derive(Validator)]
struct NewUser {
    #[rule(MaxLength(100), RegEx(r"^\S+@\S+\.\S+"))]
    email: String,
    #[rule(MinMaxLength(8, 50))]
    password: String,
    #[rule(Opt(MaxRange(Utc::now())))]
    birth_date: Option<DateTime<Utc>>
}


#[test]
#[ignore]
fn validation_test_01() {
    let new_user = NewUser {
        email: "examples@examples.com".to_test_string(),
        password: "OPw$5%hJ".to_test_string(),
        birth_date: None,
    };
    assert!(new_user.check_validity().is_ok());

    let new_user = NewUser {
        email: "examples@examples.com".to_test_string(),
        password: "O".to_test_string(),
        birth_date: None,
    };
    assert!(new_user.check_validity().is_err()); //Value is too short

    // to see error message
    new_user.check_validity().test_unwrap();
}
