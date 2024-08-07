extern crate core;

// mod test_common;

use core::any::Any;
use core::fmt;
use std::env;
use std::fmt::write;
use std::io::Write;
use mvv_common::{ make_currency, make_currency_b };
use mvv_common::entity::currency::{Currency, make_currency_b, make_currency, CurrencyFormatError };
use mvv_common::entity::currency::predefined::{EUR, USD, };
use mvv_common::result::{ as_printable, as_printable_ptr };
// use mvv_common::unchecked::UncheckedResultUnwrap;
use mvv_common::result::PrintableResult;
use mvv_common::test::{ TestDisplayStringOps, TestResultUnwrap };


#[test]
#[ignore]
fn test_temp() {

    let cur: Currency = make_currency!("USD");
    // assert_eq!(cur.code_as_string(), "USD");
    assert_eq!(cur.to_test_string(), "USD");

    // let cur: Currency = make_currency2!("US");
    // assert_eq!(cur.code_as_string(), "US");

    // const CUR43: Currency = make_currency!("usd"); // fails at compile time  - very-very GOOD
    // assert_eq!(CUR43.code_as_string(), "USD");

    // const CUR43: Currency = make_currency!("US");  // fails at compile time  - very-very GOOD
    // assert_eq!(CUR43.code_as_string(), "USD");

    let cur43: Currency = make_currency!("usd"); // Does NOT fail at compile time (only at runtime) - BAD!!! Why??
    // assert_eq!(cur43.code_as_string(), "USD");
    assert_eq!(cur43.to_test_string(), "USD");

    //const CUR44: Currency = make_currency2!("US");
    //assert_eq!(CUR44.code_as_string(), "US");

    // const CUR44: Currency = make_currency3!("US");
    // assert_eq!(CUR44.code_as_string(), "US");

    let cur45_4: Currency = make_currency!("usd");
    // assert_eq!(cur45_4.code_as_string(), "US");
    assert_eq!(cur45_4.to_test_string(), "US");

    // noinspection RsConstNaming
    // const cur45_2: Currency = make_currency("US"); // fails at compile time - vey good
    // assert_eq!(cur45_2.code_as_string(), "US");

    let not_direct_literal = "US";
    let cur45_0: Currency = make_currency(not_direct_literal);
    // let cur45: Currency = make_currency3!(not_direct_literal);
    // assert_eq!(cur45_0.code_as_string(), "US");
    assert_eq!(cur45_0.to_test_string(), "US");

    let cur46: Currency = make_currency!("US");
    // assert_eq!(cur46.code_as_string(), "US");
    assert_eq!(cur46.to_test_string(), "US");
}

// ??? Does not work !!!
//#[setup]
// pub fn setup() {
//     // setup code specific to your library's tests would go here
//     println!("### setup()");
// }

#[test]
fn make_currency_test() {
    const UAH: Currency = make_currency("UAH");
    // assert_eq!(UAH.code_as_string(), "UAH");
    assert_eq!(UAH.to_test_string(), "UAH");
    assert_eq!(UAH.code_as_ascii_bytes(), *b"UAH");
    assert_eq!(UAH.code_as_ascii_bytes(), ['U' as u8, 'A' as u8, 'H' as u8]);

    let jpy = make_currency("JPY");
    // assert_eq!(jpy.code_as_string(), "JPY");
    assert_eq!(jpy.to_test_string(), "JPY");
    assert_eq!(jpy.code_as_ascii_bytes(), *b"JPY");
    assert_eq!(jpy.code_as_ascii_bytes(), ['J' as u8, 'P' as u8, 'Y' as u8]);
}

#[test]
fn make_currency_b_test() {
    const UAH: Currency = make_currency_b(b"UAH");
    // assert_eq!(UAH.code_as_string(), "UAH");
    assert_eq!(UAH.to_test_string(), "UAH");
    assert_eq!(UAH.code_as_ascii_bytes(), *b"UAH");
    assert_eq!(UAH.code_as_ascii_bytes(), ['U' as u8, 'A' as u8, 'H' as u8]);

    //const UAH2: Currency = make_currency!(b"UAH");
    const UAH2: Currency = make_currency_b!(b"UAH");
    // assert_eq!(UAH2.code_as_string(), "UAH");
    assert_eq!(UAH2.to_test_string(), "UAH");
    assert_eq!(UAH2.code_as_ascii_bytes(), *b"UAH");
    assert_eq!(UAH2.code_as_ascii_bytes(), ['U' as u8, 'A' as u8, 'H' as u8]);

    let jpy = make_currency_b(b"JPY");
    // assert_eq!(jpy.code_as_string(), "JPY");
    assert_eq!(jpy.to_test_string(), "JPY");
    assert_eq!(jpy.code_as_ascii_bytes(), *b"JPY");
}

#[test]
fn make_currency2_test() {
    const UAH: Currency = make_currency("UAH");
    // assert_eq!(UAH.code_as_string(), "UAH");
    assert_eq!(UAH.to_test_string(), "UAH");
    assert_eq!(UAH.code_as_ascii_bytes(), ['U' as u8, 'A' as u8, 'H' as u8]);

    let jpy = make_currency("JPY");
    // assert_eq!(jpy.code_as_string(), "JPY");
    assert_eq!(jpy.to_test_string(), "JPY");
    assert_eq!(jpy.code_as_ascii_bytes(), ['J' as u8, 'P' as u8, 'Y' as u8]);
}

#[test]
fn currency_new_test() {
    let uah = Currency::new("UAH".to_test_string()).test_unwrap();
    // assert_eq!(uah.code_as_string(), "UAH");
    assert_eq!(uah.to_test_string(), "UAH");
    assert_eq!(uah.code_as_ascii_bytes(), ['U' as u8, 'A' as u8, 'H' as u8]);
}

#[test]
#[should_panic(expected = "called `Result::unwrap()` on an `Err` value: CurrencyFormatError")]
fn currency_with_wrong() {
    Currency::from_str("uAH").test_unwrap();
}

#[test]
#[should_panic(expected = "Invalid currency \"uAH\" (It should be 3 UPPERCASE english letters).")]
fn currency_new_macro_with_wrong() {
    make_currency!("uAH");
}

#[test]
fn currency_with_wrong_02() {
    let cur = Currency::from_str("uAH");
    assert!(cur.is_err());
}

#[test]
#[should_panic] // just example without message
fn impossible_make_wrong_const_literal_currency_for_non_alpha() {
    // As expected, due to 'const' var qualifier we have compilation error
    //const cur1: Currency = make_currency("1US");
    make_currency("1US");
}

#[test]
#[should_panic] // just example without message
fn macro_impossible_make_wrong_const_literal_currency_for_non_alpha() {
    // As expected, due to 'const' var qualifier we have compilation error
    //const cur1: Currency = make_currency("1US");
    make_currency!("1US");
}

#[test]
#[should_panic(expected = "Invalid currency (It should be 3 UPPERCASE english letters).")]
fn impossible_make_wrong_const_literal_currency_for_wrong_length() {
    make_currency("US");
    make_currency("USDD");
}

#[test]
#[should_panic(expected = "Invalid currency \"US\" (It should be 3 UPPERCASE english letters).")]
fn macro_impossible_make_wrong_const_literal_currency_for_wrong_length() {
    make_currency!("US");
    // make_currency!("USDD");
}

#[test]
fn impossible_make_wrong_const_literal_currency_for_non_lowercase() {
    // make_currency("usd");
    let result: Result<Currency, Box<dyn Any + Send + 'static>> = std::panic::catch_unwind(|| make_currency("usd"));
    assert!(result.is_err());

    let err: Box<dyn Any + Send> = result.unwrap_err();

    // You're more likely to want this:
    let err_type_id = err.type_id();
    println!("### err_type_id: {:?}", err_type_id);

    // type_name() can be used only if type exists (it is generic function)
    // let err_type_name = err.as_ref().type_name();
    // println!("### err_type_id: {:?}", err_type_name);

    let _as_currency_format_error = err.downcast_ref::<CurrencyFormatError>(); // None
    let _as_string = err.downcast_ref::<String>(); // None
    let as_str = err.downcast_ref::<&str>();

    println!("### err str: {:?}", as_str);

    // assert!(false, "Test error to see stdout");
}

/*
// see https://docs.rs/test-case/3.3.1/test_case/index.html#
#[test_case::test_case(-2, -4 ; "when both operands are negative")]
#[test_case::test_case(2,  4  ; "when both operands are positive")]
#[test_case::test_case(4,  2  ; "when operands are swapped")]
fn multiplication_tests(x: i8, y: i8) {
    let actual = (x * y).abs();
    assert_eq!(8, actual)
}

// see https://docs.rs/test-case/3.3.1/test_case/index.html#
#[test_case::test_case(0 => panics)]
#[test_case::test_case(1)]
fn test_divisor(divisor: usize) {
    let _result = 1 / divisor;
}
*/

// #[test]
// fn impossible_make_wrong_const_literal_currency__non_lowercase_2() {
//     // fluent-asserter library, version 0.1.7
//     // https://github.com/dmoka/fluent-asserter
//     // Seems it is not developing now :-(
//     //
//     assert_that_code!(|| make_currency("usd"))
//         .panics()
//         .with_message("some panic message");
// }


#[test]
#[allow(unused_mut)]
fn impossible_to_change_existent_currency_from_outside_package() {

    let mut temp_obj: Currency = USD;
    temp_obj.code_as_ascii_bytes()[0] = 'W' as u8;
    // temp_obj.code_as_string().push('Z'); // As expected it does not change currency object.
    temp_obj.to_test_string().push('Z'); // As expected it does not change currency object.
    println!("{}", temp_obj);

    // assert_eq!(temp_obj.code_as_string(), "USD");
    assert_eq!(temp_obj.to_test_string(), "USD");
}

#[test]
fn use_public_constants() {
    // assert_eq!(USD.code_as_string(), "USD");
    assert_eq!(USD.to_test_string(), "USD");
    assert_eq!(EUR.code_as_ascii_bytes()[0], 'E' as u8);
    assert_eq!(EUR.code_as_ascii_bytes()[1], 'U' as u8);
    assert_eq!(EUR.code_as_ascii_bytes()[2], 'R' as u8);
}

#[test]
fn test_currency_eq() {
    assert_eq!(Currency::from_str("USD").test_unwrap(), Currency::from_str("USD").test_unwrap());
    assert_ne!(Currency::from_str("USD").test_unwrap(), Currency::from_str("EUR").test_unwrap());
}


#[test]
fn test_main_1() {
    println!("Hello, world!");

    println!("Current dir: {:?}", env::current_dir());
    println!("std::env MY_VAR1 = {:?}", env::var("MY_VAR1"));

    dotenv::dotenv().ok();
    println!("std::env MY_VAR1 = {:?}", env::var("MY_VAR1"));

    println!("dotenv::var MY_VAR1 = {:?}", dotenv::var("MY_VAR1"));
}



#[test]
fn test_main_2() {
    let usd_literal: &str = "USD";
    let _usd_str: String = usd_literal.to_test_string();
    let _usd_bytes: &[u8] = usd_literal.as_bytes();

    println!("{}", USD);
    //println!("{}", USD.code());
    // println!("{} (as code_as_string)", USD.code_as_string());
    println!("{} (as string)", USD.to_test_string());

    // let cur2 = Currency::new(['u' as u8, 'S' as u8, 'D' as u8]);
    let cur2: Result<Currency, CurrencyFormatError> = Currency::new("uSD".to_test_string());
    //println!("{}", cur2);
    println!("{} (as PrintableResult)", PrintableResult(&cur2));
    println!("{} (as_printable)", as_printable(&cur2));
    println!("{} (as_printable2)", as_printable_ptr(&cur2));

    // let cur2 = Currency::new(['u' as u8, 'S' as u8, 'D' as u8]);
    let cur2: Result<Currency, CurrencyFormatError> = Currency::from_str("BRL");
    //println!("{}", cur2);
    println!("{} (as PrintableResult)", PrintableResult(&cur2));
    println!("{} (as_printable)", as_printable(&cur2));
    println!("{} (as_printable2)", as_printable_ptr(&cur2));
    // let currency1: Currency = currency_from_string(&usd_str).expect("Fuck happened.");
    // println!("{}", currency1.to_test_string());
    //
    // let currency: Currency = currency_from_bytes(usd_bytes).expect("Fuck happened.");
    // println!("{}", currency.to_test_string());
    //
    // let currency: Currency = currency_from_str("EUR").expect("Fuck happened.");
    // println!("{}", currency.to_test_string());


    let mut output = String::new();
    if let Err(fmt::Error) = write(&mut output, format_args!("Hello {}!", "world\n")) {
        println!("An error occurred");
        panic!("An error occurred");
    }

    let buf: &[u8] = output.as_str().as_bytes();
    std::io::stdout().write(buf).test_unwrap();
    std::io::stdout().flush().test_unwrap();
}

/*
#[allow(dead_code)]
fn use_dead_code() {
    let usd_code_as_bytes = USD.code_as_ascii_bytes();
    println!("{}", usd_code_as_bytes[0]);

    let usd_code_as_string = currency::EUR.code_as_string();
    println!("{}", usd_code_as_string);

    let usd = Currency::new("USD".to_test_string());
    println!("{}", usd.u n w r a p());

    let brl = currency::make_currency("BRL");
    println!("{}", brl);
}
*/

#[allow(dead_code)]
fn factorial(n: u128) -> u128 {
    match n {
        0 => 1,
        n => n * factorial(n - 1),
    }
}
