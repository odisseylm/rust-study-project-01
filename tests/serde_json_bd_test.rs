use bigdecimal::BigDecimal;
use serde::{ Deserialize, Serialize };
use project01::util::TestResultUnwrap;

#[derive(Debug, Serialize, Deserialize)]
struct StructBDSerByFn {
    #[serde(
        serialize_with = "project01::entities::serde_json_bd::serialize_json_bd",
        deserialize_with = "project01::entities::serde_json_bd::deserialize_json_bd",
    )]
    bd: BigDecimal,
}

#[test]
fn test_big_decimal_fields_as_ser_by_fn() {
    use core::str::FromStr;

    let s = StructBDSerByFn { bd: BigDecimal::from_str("12.345").test_unwrap() };

    let s = serde_json::to_string(&s).test_unwrap();
    println!("###s: {}", s);

    assert_eq!(s, r#"{"bd":12.345}"#);

    let s: StructBDSerByFn = serde_json::from_str(r#"{"bd":13.346}"#).test_unwrap();
    assert_eq!(s.bd, BigDecimal::from_str("13.346").test_unwrap());
}


#[derive(Debug, Serialize, Deserialize)]
struct StructBDWithSerModule {
    #[serde(with = "project01::entities::serde_json_bd::bd_with")]
    bd: BigDecimal,
}

#[test]
fn test_big_decimal_fields_as_with_ser_module() {
    use core::str::FromStr;

    let s = StructBDWithSerModule { bd: BigDecimal::from_str("12.345").test_unwrap() };

    let s = serde_json::to_string(&s).test_unwrap();
    println!("###s: {}", s);

    assert_eq!(s, r#"{"bd":12.345}"#);

    let s: StructBDWithSerModule = serde_json::from_str(r#"{"bd":13.346}"#).test_unwrap();
    assert_eq!(s.bd, BigDecimal::from_str("13.346").test_unwrap());
}



#[derive(Debug, Serialize, Deserialize)]
struct StructBDSerNewTypeWrapper {
    #[serde(
        serialize_with = "project01::entities::serde_json_bd::serialize_json_bd",
        deserialize_with = "project01::entities::serde_json_bd::deserialize_json_bd",
    )]
    bd: BigDecimal,
}


// TODO: add precision & from string & to string serialization/deserialization.
