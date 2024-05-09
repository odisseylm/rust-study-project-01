
use std::str::FromStr;
use bigdecimal::BigDecimal;
use project01::entities::{Amount, amount};
use project01::entities::currency::{ EUR, USD, make_currency };
use project01::make_currency;


mod common;


// private for testing
fn bd(v: &str) -> BigDecimal { BigDecimal::from_str(v).unwrap() }


#[test]
fn amount_create() {
    let amount1 = Amount::new(bd("123.456"), USD);
    assert_eq!(amount1.to_string(), "123.456 USD");

    let amount1 = amount(bd("124.456"), EUR);
    assert_eq!(amount1.to_string(), "124.456 EUR");

    assert_eq!(Amount::with_str_amount("10.050", USD).unwrap().to_string(), "10.050 USD");

    let amount_string: String = "10.0501".to_string();
    assert_eq!(Amount::with_str_amount(amount_string.as_str(), USD).unwrap().to_string(), "10.0501 USD");

    // assert_eq!(Amount::with_str_amount("10.050", USD).unwrap().to_string(), "10.050 USD");
    // assert_eq!(Amount::with_string_amount2("10.0501".to_string(), USD).unwrap().to_string(), "10.0501 USD");
}


#[test]
fn amount_base_test() {
    let am = Amount::with_str_amount("10.050", USD);

    let amount = am.unwrap();
    assert_eq!(amount.to_string(), "10.050 USD");

    // assert_eq!(amount.value.to_string(), "10.050");
    // assert_eq!(amount.value.to_string(), "10.050");

    assert_eq!(amount.value().to_string(), "10.050");
    assert_eq!(amount.value().to_string(), "10.050");

    // repeatable calls, to test borrowing/moving
    assert_eq!(amount.value_ref().to_string(), "10.050");
    assert_eq!(amount.value_ref().to_string(), "10.050");

    assert_eq!(amount.currency().to_string(), "USD");
}


#[test]
fn amount_should_be_immutable() {
    let amount = Amount::with_str_amount_unchecked("10.050", EUR);

    assert_eq!(amount.to_string(), "10.050 EUR");

    // amount.currency() = EUR; // compilation error 'invalid left-hand side of assignment' - OK
    // assert_eq!(amount.to_string(), "10.050 EUR"); // not changed

    // amount.value() = &bd("22.022"); // compilation error 'invalid left-hand side of assignment' - OK
    // assert_eq!(amount.to_string(), "10.050 EUR"); // not changed

    // amount.value_ref() = bd("22.022").to_ref(); // compilation error 'invalid left-hand side of assignment' - OK
    // assert_eq!(amount.to_string(), "10.050 EUR"); // not changed

    amount.value().inverse();
    assert_eq!(amount.to_string(), "10.050 EUR"); // not changed

    amount.with_value(bd("22.22"));
    assert_eq!(amount.to_string(), "10.050 EUR"); // not changed

    // amount.with_currency(USD);
    // assert_eq!(amount.to_string(), "10.050 EUR"); // not changed
}


#[test]
#[allow(unused_mut)]
fn amount_mutability_test() {
    let am = Amount::with_str_amount("10.050", make_currency!("JPY"));

    let mut amount = am.unwrap();
    assert_eq!(amount.to_string(), "10.050 JPY");

    // amount.currency() = EUR; // compilation error 'invalid left-hand side of assignment' - OK
    // assert_eq!(amount.to_string(), "10.050 JPY"); // not changed

    // amount.value() = &bd("22.022"); // compilation error 'invalid left-hand side of assignment' - OK
    // assert_eq!(amount.to_string(), "10.050 JPY"); // not changed

    // amount.value_ref() = bd("22.022").to_ref(); // compilation error 'invalid left-hand side of assignment' - OK
    // assert_eq!(amount.to_string(), "10.050 JPY"); // not changed

    amount.value().inverse();
    assert_eq!(amount.to_string(), "10.050 JPY"); // not changed

    let new_amount = amount.with_value(bd("22.22"));
    assert_eq!(amount.to_string(), "10.050 JPY"); // not changed
    assert_eq!(new_amount.to_string(), "22.22 JPY"); // not changed

    // let new_amount = amount.with_currency(make_currency!("BRL"));
    // assert_eq!(amount.to_string(), "10.050 JPY"); // not changed
    // assert_eq!(new_amount.to_string(), "10.050 BRL"); // not changed
}


#[test]
fn from_string() {
    let am = Amount::from_str(" \t \n 122.350  \tJPY ").unwrap();
    assert_eq!(am.to_string(), "122.350 JPY");
    assert_eq!(*am.value(), bd("122.350"));
    assert_eq!(am.value_ref(), bd("122.350").to_ref());
    assert_eq!(am.currency(), make_currency!("JPY"));
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: ParseCurrencyError")]
fn from_string_with_wrong_formed_currency() {
    let am = Amount::from_str(" \t \n 122.350 USSD ").unwrap();
    assert_eq!(am.to_string(), "122.350 JPY");
    assert_eq!(*am.value(), bd("122.350"));
    assert_eq!(am.value_ref(), bd("122.350").to_ref());
    assert_eq!(am.currency(), make_currency!("JPY"));
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: NoCurrencyError")]
fn from_string_without_currency() {
    let am = Amount::from_str(" \t \n 122.350  ").unwrap();
    assert_eq!(am.to_string(), "122.350 JPY");
    assert_eq!(*am.value(), bd("122.350"));
    assert_eq!(am.value_ref(), bd("122.350").to_ref());
    assert_eq!(am.currency(), make_currency!("JPY"));
}

#[test]
// #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: ParseBigInt(ParseBigIntError { kind: InvalidDigit })")]
#[should_panic(expected = "ParseBigInt(ParseBigIntError { kind: InvalidDigit })")]
fn from_string_with_wrong_amount_value() {
    let am = Amount::from_str(" \t \n 12_John_2.350 BRL ").unwrap();
    assert_eq!(am.to_string(), "122.350 JPY");
    assert_eq!(*am.value(), bd("122.350"));
    assert_eq!(am.value_ref(), bd("122.350").to_ref());
    assert_eq!(am.currency(), make_currency!("JPY"));
}

#[test]
// #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: ParseBigInt(ParseBigIntError { kind: InvalidDigit })")]
#[should_panic(expected = "ParseBigInt(ParseBigIntError { kind: InvalidDigit })")]
fn from_string_with_wrong_non_ascii_amount_value() {
    let am = Amount::from_str(" \t \n Чебуран BRL ").unwrap();
    assert_eq!(am.to_string(), "122.350 JPY");
    assert_eq!(*am.value(), bd("122.350"));
    assert_eq!(am.value_ref(), bd("122.350").to_ref());
    assert_eq!(am.currency(), make_currency!("JPY"));
}
