use assertables::{ assert_in_epsilon, assert_in_epsilon_as_result };
use bigdecimal::BigDecimal;
use serde::{ Deserialize, Serialize };
use mvv_common::test::{ TestOptionUnwrap, TestResultUnwrap };


#[derive(Debug, Serialize, Deserialize)]
struct StructBDSerByFn {
    #[serde(
        serialize_with = "mvv_common::json::serde_json_bd::serialize_json_bd",
        deserialize_with = "mvv_common::json::serde_json_bd::deserialize_json_bd",
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
struct StructBDSerByStrFn {
    #[serde(
        serialize_with = "mvv_common::json::serde_json_bd::serialize_json_bd_as_string",
        deserialize_with = "mvv_common::json::serde_json_bd::deserialize_json_bd_as_std_json_value",
    )]
    bd: BigDecimal,
}

#[test]
fn test_big_decimal_fields_as_ser_by_str_fn() {
    use core::str::FromStr;

    let s = StructBDSerByStrFn { bd: BigDecimal::from_str("12.345").test_unwrap() };

    let s = serde_json::to_string(&s).test_unwrap();
    println!("###s: {}", s);

    assert_eq!(s, r#"{"bd":"12.345"}"#);

    let s: StructBDSerByStrFn = serde_json::from_str(r#"{"bd":"13.346"}"#).test_unwrap();
    assert_eq!(s.bd, BigDecimal::from_str("13.346").test_unwrap());
}


#[derive(Debug, Serialize, Deserialize)]
struct StructBDDeSerByF64 {
    #[serde(
        serialize_with = "mvv_common::json::serde_json_bd::serialize_json_bd_as_f64",
        deserialize_with = "mvv_common::json::serde_json_bd::deserialize_json_bd_as_std_json_value",
    )]
    bd: BigDecimal,
}

#[test]
fn test_big_decimal_fields_as_ser_by_f64_fn() {
    use core::str::FromStr;

    let s = StructBDDeSerByF64 { bd: BigDecimal::from_str("12.345").test_unwrap() };

    let s = serde_json::to_string(&s).test_unwrap();
    println!("###s: {}", s);

    assert_eq!(s, r#"{"bd":12.345}"#);

    use bigdecimal::{ FromPrimitive, ToPrimitive };

    // !!! BigDecimal has very-very BAD converting from f32/f64 !!!
    // If value is not nan/infinity/etc, it is better to convert f32/f64 to string,
    // and convert resulting string to BigDecimal
    let bd_from_f64 = BigDecimal::from_f64(13.346f64).test_unwrap();
    let bd_from_str = BigDecimal::from_str("13.346").test_unwrap();
    println!("bd_from_str: {}, bd_from_f64: {}", bd_from_str, bd_from_f64);
    assert_in_epsilon!(bd_from_str.to_f64().test_unwrap(), bd_from_f64.to_f64().test_unwrap(), 0.00000000000001);
    // assert_eq!(bd_from_f64, bd_from_str);

    let s: StructBDDeSerByF64 = serde_json::from_str(r#"{"bd":15.346}"#).test_unwrap();
    assert_eq!(s.bd, BigDecimal::from_str("15.346").test_unwrap());
    // assert_in_epsilon!(
    //     s.bd.to_f64().test_unwrap(),
    //     BigDecimal::from_str("15.346").unwrap().to_f64().test_unwrap(),
    //     0.000000000000001);
}


#[derive(Debug, Serialize, Deserialize)]
struct StructBDWithSerModule {
    #[serde(with = "mvv_common::json::serde_json_bd::bd_with")]
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


#[cfg(feature = "serde_json_raw_value")]
#[derive(Debug, Serialize, Deserialize)]
struct StructBDDeSerByRawValue {
    #[serde(
        serialize_with = "mvv_common::json::serde_json_bd::serialize_json_bd_as_raw_value",
        deserialize_with = "mvv_common::json::serde_json_bd::deserialize_json_bd_as_raw_value",
    )]
    bd: BigDecimal,
}

#[cfg(feature = "serde_json_raw_value")]
#[test]
fn test_big_decimal_fields_as_ser_by_raw_value() {
    use core::str::FromStr;

    let s = StructBDDeSerByRawValue { bd: BigDecimal::from_str(
        "12.345111111111111111222222222222222233333333333334444444444455555555").test_unwrap() };

    let s = serde_json::to_string(&s).test_unwrap();
    println!("###s: {}", s);
    assert_eq!(s, r#"{"bd":12.345111111111111111222222222222222233333333333334444444444455555555}"#);

    let s: StructBDDeSerByRawValue = serde_json::from_str(
        r#"{"bd":12.345111111111111111222222222222222233333333333334444444444455555555}"#).test_unwrap();
    assert_eq!(s.bd, BigDecimal::from_str("12.345111111111111111222222222222222233333333333334444444444455555555").test_unwrap());
}
