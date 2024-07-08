mod common;

use core::fmt::write;
use core::str::FromStr;
use bigdecimal::BigDecimal;

use project01::entities::amount::{ Amount, amount };
use project01::entities::amount::parse;
use project01::entities::amount::parse::{ ParseAmountError };
use project01::entities::currency::{ EUR, USD, make_currency };
use project01::make_currency;

use common::{ TestOptionUnwrap, TestResultUnwrap };

use assertables::{ assert_contains, assert_contains_as_result };
use assertables::{ assert_starts_with, assert_starts_with_as_result };
use project01::util::backtrace::is_anyhow_backtrace_enabled;
use project01::util::enable_backtrace;
use project01::util::test_unwrap::TestSringOps;


// private for testing
fn bd(v: &str) -> BigDecimal { BigDecimal::from_str(v).test_unwrap() }


#[test]
fn amount_create() {
    let amount1 = Amount::new(bd("123.456"), USD);
    assert_eq!(amount1.to_test_string(), "123.456 USD");

    let amount1 = amount(bd("124.456"), EUR);
    assert_eq!(amount1.to_test_string(), "124.456 EUR");

    assert_eq!(Amount::with_str_amount("10.050", USD).test_unwrap().to_test_string(), "10.050 USD");

    let amount_string: String = "10.0501".to_test_string();
    assert_eq!(Amount::with_str_amount(amount_string.as_str(), USD).test_unwrap().to_test_string(), "10.0501 USD");

    // assert_eq!(Amount::with_str_amount("10.050", USD).test_unwrap().to_test_string(), "10.050 USD");
    // assert_eq!(Amount::with_string_amount2("10.0501".to_test_string(), USD).test_unwrap().to_test_string(), "10.0501 USD");
}


#[test]
fn amount_base_test() {
    let am = Amount::with_str_amount("10.050", USD);

    let amount = am.test_unwrap();
    assert_eq!(amount.to_test_string(), "10.050 USD");

    // assert_eq!(amount.value.to_test_string(), "10.050");
    // assert_eq!(amount.value.to_test_string(), "10.050");

    assert_eq!(amount.value_ref().to_test_string(), "10.050");
    assert_eq!(amount.value_ref().to_test_string(), "10.050");

    // repeatable calls, to test borrowing/moving
    assert_eq!(amount.value_bd_ref().to_test_string(), "10.050");
    assert_eq!(amount.value_bd_ref().to_test_string(), "10.050");

    assert_eq!(amount.currency().to_test_string(), "USD");
}


#[test]
fn amount_should_be_immutable() {
    let amount = Amount::with_str_amount_unchecked("10.050", EUR);

    assert_eq!(amount.to_test_string(), "10.050 EUR");

    // amount.currency() = EUR; // compilation error 'invalid left-hand side of assignment' - OK
    // assert_eq!(amount.to_test_string(), "10.050 EUR"); // not changed

    // amount.value() = &bd("22.022"); // compilation error 'invalid left-hand side of assignment' - OK
    // assert_eq!(amount.to_test_string(), "10.050 EUR"); // not changed

    // amount.value_ref() = bd("22.022").to_ref(); // compilation error 'invalid left-hand side of assignment' - OK
    // assert_eq!(amount.to_test_string(), "10.050 EUR"); // not changed

    amount.value_ref().inverse();
    assert_eq!(amount.to_test_string(), "10.050 EUR"); // not changed

    amount.with_value(bd("22.22"));
    assert_eq!(amount.to_test_string(), "10.050 EUR"); // not changed

    // amount.with_currency(USD);
    // assert_eq!(amount.to_test_string(), "10.050 EUR"); // not changed
}


#[test]
#[allow(unused_mut)]
fn amount_mutability_test() {
    let am = Amount::with_str_amount("10.050", make_currency!("JPY"));

    let mut amount = am.test_unwrap();
    assert_eq!(amount.to_test_string(), "10.050 JPY");

    // amount.currency() = EUR; // compilation error 'invalid left-hand side of assignment' - OK
    // assert_eq!(amount.to_test_string(), "10.050 JPY"); // not changed

    // amount.value() = &bd("22.022"); // compilation error 'invalid left-hand side of assignment' - OK
    // assert_eq!(amount.to_test_string(), "10.050 JPY"); // not changed

    // amount.value_ref() = bd("22.022").to_ref(); // compilation error 'invalid left-hand side of assignment' - OK
    // assert_eq!(amount.to_test_string(), "10.050 JPY"); // not changed

    amount.value_ref().inverse();
    assert_eq!(amount.to_test_string(), "10.050 JPY"); // not changed

    let new_amount = amount.with_value(bd("22.22"));
    assert_eq!(amount.to_test_string(), "10.050 JPY"); // not changed
    assert_eq!(new_amount.to_test_string(), "22.22 JPY"); // not changed

    // let new_amount = amount.with_currency(make_currency!("BRL"));
    // assert_eq!(amount.to_test_string(), "10.050 JPY"); // not changed
    // assert_eq!(new_amount.to_test_string(), "10.050 BRL"); // not changed
}


#[test]
fn from_string() {
    let am = Amount::from_str(" \t \n 122.350  \tJPY ").test_unwrap();
    assert_eq!(am.to_test_string(), "122.350 JPY");
    assert_eq!(*am.value_ref(), bd("122.350"));
    assert_eq!(am.value_bd_ref(), bd("122.350").to_ref());
    assert_eq!(am.currency(), make_currency!("JPY"));
}

#[test]
// #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: ParseCurrencyError")]
// #[should_panic(expected = "`Err` value: ParseAmountError { kind: ParseCurrencyError { source: CurrencyFormatError { kind: CurrencyFormatError")]
// #[should_panic(expected = "`Err` value: ParseAmountError { kind: ParseCurrencyError, source: CurrencyFormatError(CurrencyFormatError { kind: CurrencyFormatError")]
// #[should_panic(expected = "`Err` value: ParseAmountError { kind: ParseCurrencyError, source: CurrencyFormatError { kind: CurrencyFormatError")]
// #[should_panic(expected = "`Err` value: ParseAmountError { kind: IncorrectCurrency, source: CurrencyFormatError { kind: CurrencyFormatError")]
#[should_panic(expected = "`Err` value: ParseAmountError { kind: IncorrectCurrency, source: CurrencyFormatError { kind: IncorrectCurrencyFormat")]
fn from_string_with_wrong_formed_currency() {
    enable_backtrace();
    Amount::from_str(" \t \n 122.350 USSD ").test_unwrap();
}

#[test]
fn from_string_with_wrong_formed_currency_02() {
    enable_backtrace();
    let res = Amount::from_str(" \t \n 122.350 USSD ");
    if let Err(ref err) = res {

        println!("--------------------------------------------------------------------------------");
        println!("err (display): {}", err);
        println!("--------------------------------------------------------------------------------");
        println!("err (debug)  : {:?}", err);

        println!("--------------------------------------------------------------------------------");
        println!("err.source (display): {}", err.source);
        println!("--------------------------------------------------------------------------------");
        println!("err.source (debug)  : {:?}", err.source);
    }
}

#[test]
fn from_string_with_wrong_formed_currency_do_not_print_stack_trace_twice() {
    enable_backtrace();
    let res = Amount::from_str(" \t \n 122.350 USSD ");
    if let Err(ref err) = res {

        println!("--------------------------------------------------------------------------------");
        println!("err (display): {}", err);
        println!("--------------------------------------------------------------------------------");
        println!("err (debug)  : {:?}", err);

        println!("--------------------------------------------------------------------------------");
        println!("err.source (display): {}", err.source);
        println!("--------------------------------------------------------------------------------");
        println!("err.source (debug)  : {:?}", err.source);

        // assert_display_stack_trace_is_only_one(&err);
        assert_debug_stack_trace_is_only_one(&err);

        // assert_display_stack_trace_is_only_one(&err.source);
        assert_debug_stack_trace_is_only_one(&err.source);

        if let parse::ErrorSource::CurrencyFormatError(ref err) = err.source {

            println!("--------------------------------------------------------------------------------");
            println!("err (display): {}", err);
            println!("--------------------------------------------------------------------------------");
            println!("err (debug)  : {:?}", err);

            // println!("--------------------------------------------------------------------------------");
            // println!("err.source (display): {}", err.source);
            // println!("--------------------------------------------------------------------------------");
            // println!("err.source (debug)  : {:?}", err.source);

            // assert_display_stack_trace_is_only_one(&err);
            assert_display_no_stack_trace(&err);
            assert_debug_stack_trace_is_only_one(&err);

            // assert_display_stack_trace_is_only_one(&err.source);
            // assert_debug_stack_trace_is_only_one(&err.source);
        } else {
            assert!(false, "Unexpected flow.")
        }
    }
}

#[test]
fn from_string_with_wrong_amount_value_do_not_print_stack_trace_twice() {
    enable_backtrace();
    let res = Amount::from_str(" \t \n John_350 USD ");
    if let Err(ref err) = res {

        println!("--------------------------------------------------------------------------------");
        println!("err (display): {}", err);
        println!("--------------------------------------------------------------------------------");
        println!("err (debug)  : {:?}", err);

        println!("--------------------------------------------------------------------------------");
        println!("err.source (display): {}", err.source);
        println!("--------------------------------------------------------------------------------");
        println!("err.source (debug)  : {:?}", err.source);

        // assert_display_stack_trace_is_only_one(&err);
        assert_display_no_stack_trace(&err);
        assert_debug_stack_trace_is_only_one(&err);

        // assert_display_stack_trace_is_only_one(&err.source);
        // T O D O: why no stack trace? Is it ok?
        // assert_debug_stack_trace_is_only_one(&err.source);

        if let parse::ErrorSource::ParseBigDecimalError(ref err) = err.source {

            println!("--------------------------------------------------------------------------------");
            println!("err (display): {}", err);
            println!("--------------------------------------------------------------------------------");
            println!("err (debug)  : {:?}", err);

            // println!("--------------------------------------------------------------------------------");
            // println!("err.source (display): {}", err.source);
            // println!("--------------------------------------------------------------------------------");
            // println!("err.source (debug)  : {:?}", err.source);

            // if it fails in the future we will need to verify other 'print' backtrace cases
            assert_display_no_stack_trace(&err);
            assert_debug_no_stack_trace(&err);

            // assert_display_no_stack_trace(&err.source);
            // assert_debug_no_stack_trace(&err.source);
        } else {
            assert!(false, "Unexpected flow.")
        }
    }
}


#[track_caller]
#[allow(dead_code)]
fn assert_display_stack_trace_is_only_one<Err: core::fmt::Display>(err: &Err) {
    use core::fmt::Write;
    let mut str_buf = String::new();
    write!(str_buf, "{}", err).test_unwrap();
    assert_stack_trace_is_only_one(str_buf.as_str());
}
#[track_caller]
#[allow(dead_code)]
fn assert_debug_stack_trace_is_only_one<Err: core::fmt::Debug>(err: &Err) {
    use core::fmt::Write;
    let mut str_buf = String::new();
    write!(str_buf, "{:?}", err).test_unwrap();
    assert_stack_trace_is_only_one(str_buf.as_str());
}
#[track_caller]
fn assert_stack_trace_is_only_one(str: &str) {
    let first_index: Option<usize> = str.find("backtrace:")
        .or_else(|| str.find("stacktrace:"))
        .or_else(|| str.find("stack trace:"))
        ;

    assert!(first_index.is_some(), "No any backtrace is found in [{}]", str);

    let second_index: Option<usize> = first_index.and_then(|first_index| {
        let str: &str = &str[first_index + 1..];

        str.find("backtrace:")
            .or_else(|| str.find("stacktrace:"))
            .or_else(|| str.find("stack trace:"))
    });

    assert!(second_index.is_some(), "No any backtrace is found in [{}]", str);
}


#[test]
fn aa() {
    enable_backtrace();
    let res = Amount::from_str(" \t \n John_350 USD ");
    if let Err(ref err) = res {

        println!("err.source (display): {}", err.source);
        println!("err.source (debug)  : {:?}", err.source);

        if let parse::ErrorSource::ParseBigDecimalError(ref bd_err) = err.source {
            println!("err.source (display): {}", bd_err);
            println!("err.source (debug)  : {:?}", bd_err);
        }
    }
}


#[track_caller]
#[allow(dead_code)]
fn assert_display_no_stack_trace<Err: core::fmt::Display>(err: &Err) {
    use core::fmt::Write;
    let mut str_buf = String::new();
    write!(str_buf, "{}", err).test_unwrap();
    assert_no_stack_trace(str_buf.as_str());
}
#[track_caller]
#[allow(dead_code)]
fn assert_debug_no_stack_trace<Err: core::fmt::Debug>(err: &Err) {
    use core::fmt::Write;
    let mut str_buf = String::new();
    write!(str_buf, "{:?}", err).test_unwrap();
    assert_no_stack_trace(str_buf.as_str());
}
#[track_caller]
fn assert_no_stack_trace(str: &str) {
    let first_index: Option<usize> = str.find("backtrace:")
        .or_else(|| str.find("stacktrace:"))
        .or_else(|| str.find("stack trace:"))
        ;

    assert!(first_index.is_none(), "There is at least one backtrace is found (but non expected) in [{}]", str);
}


#[test]
// #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: NoCurrencyError")]
// #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: ParseAmountError { kind: NoCurrencyError")]
#[should_panic(expected = "`Err` value: ParseAmountError { kind: NoCurrency, source: NoSource")]
fn from_string_without_currency() {
    Amount::from_str(" \t \n 122.350  ").test_unwrap();
}

#[test]
// #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: ParseBigInt(ParseBigIntError { kind: InvalidDigit })")]
#[should_panic(expected = "ParseBigInt(ParseBigIntError { kind: InvalidDigit })")]
fn from_string_with_wrong_amount_value() {
    Amount::from_str(" \t \n 12_John_2.350 BRL ").test_unwrap();
}

#[test]
// #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: ParseBigInt(ParseBigIntError { kind: InvalidDigit })")]
#[should_panic(expected = "ParseBigInt(ParseBigIntError { kind: InvalidDigit })")]
fn from_string_with_wrong_non_ascii_amount_value() {
    Amount::from_str(" \t \n Чебуран BRL ").test_unwrap();
}

fn fn_test_parse_amount_01() -> Result<Amount, anyhow::Error> {
    Amount::from_str(" \t \n Чебуран BRL ").map_err(anyhow::Error::new)
}
fn fn_test_parse_amount_02() -> Result<Amount, anyhow::Error> { fn_test_parse_amount_01() }
fn fn_test_parse_amount_03() -> Result<Amount, anyhow::Error> { fn_test_parse_amount_02() }

#[test]
fn test_anyhow_stacktrace() {
    enable_backtrace();

    println!("\n*******************************************************************");
    println!("TEST test_anyhow_stacktrace\n");

    let am = fn_test_parse_amount_03();
    let err = am.err().test_unwrap();
    println!("err: {err:?}");
    println!("err: {err}");

    let mut output = String::new();
    write(&mut output, format_args!("{err:?}")).test_unwrap();

    println!("\n-------------------------------------------------------------------");
    println!("err as str:\n{}", output);
    println!("-------------------------------------------------------------------\n");

    assert_starts_with!(output, "ParseAmountError { Incorrect amount format }");

    if is_anyhow_backtrace_enabled() {
        assert_contains!(output, "Stack backtrace:");

        assert_contains!(output, "amount_test::fn_test_parse_amount_01\n             at ./tests/amount_test.rs:");
        assert_contains!(output, "amount_test::fn_test_parse_amount_02\n             at ./tests/amount_test.rs:");
        assert_contains!(output, "amount_test::fn_test_parse_amount_03\n             at ./tests/amount_test.rs:");
        assert_contains!(output, "amount_test::test_anyhow_stacktrace\n             at ./tests/amount_test.rs:");
        // it is risky/dependant
        // assert_contains!(output, "amount_test::test_anyhow_stacktrace::{{closure}}\n             at ./tests/amount_test.rs:");
    }
}


fn fn_test_parse_amount_101() -> Result<Amount, ParseAmountError> {
    Amount::from_str(" \t \n Чебуран BRL ")
}
fn fn_test_parse_amount_102() -> Result<Amount, ParseAmountError> { fn_test_parse_amount_101() }
fn fn_test_parse_amount_103() -> Result<Amount, ParseAmountError> { fn_test_parse_amount_102() }



#[test]
fn test_my_stacktrace() {
    enable_backtrace();

    println!("\n*******************************************************************");
    println!("TEST test_my_stacktrace\n");

    let am = fn_test_parse_amount_103();
    let err = am.err().test_unwrap();

    println!("err: {err:?}");

    let mut output = String::new();
    write(&mut output, format_args!("{err:?}")).test_unwrap();

    println!("\n-------------------------------------------------------------------");
    println!("err as str:\n{}", output);
    println!("-------------------------------------------------------------------\n");

    // assert_starts_with!(output, "ParseAmountError { source: ParseBigInt(ParseBigIntError { kind: InvalidDigit })");
    // assert_starts_with!(output, "ParseAmountError { kind: ParseAmountError { source: ParseBigInt(ParseBigIntError { kind: InvalidDigit }) }");
    // assert_starts_with!(output, "ParseAmountError { kind: ParseAmountError, source: ParseBigDecimalError(ParseBigInt(ParseBigIntError { kind: InvalidDigit }))");
    // assert_starts_with!(output, "ParseAmountError { kind: ParseAmountError, source: ParseBigInt(ParseBigIntError { kind: InvalidDigit })");
    assert_starts_with!(output, "ParseAmountError { kind: IncorrectAmount, source: ParseBigInt(ParseBigIntError { kind: InvalidDigit })");
    assert_contains!(output, "backtrace:");

    assert_contains!(output, "amount_test::fn_test_parse_amount_101\n             at ./tests/amount_test.rs:");
    assert_contains!(output, "amount_test::fn_test_parse_amount_102\n             at ./tests/amount_test.rs:");
    assert_contains!(output, "amount_test::fn_test_parse_amount_103\n             at ./tests/amount_test.rs:");
    assert_contains!(output, "amount_test::test_my_stacktrace\n             at ./tests/amount_test.rs:");
    // it is risky/dependant
    // assert_contains!(output, "amount_test::test_my_stacktrace::{{closure}}\n             at ./tests/amount_test.rs:");

    println!("\n--------------------------------------------------------\n");
    println!("err: {err}");
    // let backtrace = match err {
    //     ParseAmountError::NoCurrencyError { backtrace } => { backtrace }
    //     ParseAmountError::ParseCurrencyError { source:_, backtrace } => {backtrace}
    //     ParseAmountError::ParseAmountError { source:_, backtrace } => { backtrace}
    // };
    let backtrace = err.backtrace ;
    println!("my stacktrace: {}", backtrace);

    println!("\n----------------------------------------------\n");
    println!("my stacktrace as Debug");
    println!("my stacktrace backtrace_status: {:?}", backtrace.backtrace_status());
    println!("my stacktrace backtrace: {}", backtrace.backtrace());
    println!("my stacktrace backtrace: {}", backtrace.std_backtrace().test_unwrap());

    println!("\n----------------------------------------------\n");
    println!("my stacktrace as Display");
    println!("my stacktrace backtrace_status: {:?}", backtrace.backtrace_status());
    println!("my stacktrace backtrace: {:?}", backtrace.backtrace());
    println!("my stacktrace backtrace: {:?}", backtrace.std_backtrace().test_unwrap());
}



fn fn_test_parse_amount_201() -> Result<Amount, Box<dyn std::error::Error>> {
    let amount = Amount::from_str(" \t \n Чебуран BRL ") ?;
    Ok(amount)
}
fn fn_test_parse_amount_202() -> Result<Amount, Box<dyn std::error::Error>> { fn_test_parse_amount_201() }
fn fn_test_parse_amount_203() -> Result<Amount, Box<dyn std::error::Error>> { fn_test_parse_amount_202() }



#[test]
fn test_std_error() {
    enable_backtrace();

    let am = fn_test_parse_amount_203();
    let err = am.err().test_unwrap();

    println!("\n*******************************************************************");
    println!("TEST test_std_error\n");

    let mut output = String::new();
    write(&mut output, format_args!("{err:?}")).test_unwrap();

    println!("\n-------------------------------------------------------------------");
    println!("err as str:\n{}", output);
    println!("-------------------------------------------------------------------\n");

    // assert_starts_with!(output, "ParseAmountError { source: ParseBigInt(ParseBigIntError { kind: InvalidDigit })");
    // assert_starts_with!(output, "ParseAmountError { kind: ParseAmountError { source: ParseBigInt(ParseBigIntError { kind: InvalidDigit }) }");
    // assert_starts_with!(output, "ParseAmountError { kind: ParseAmountError, source: ParseBigDecimalError(ParseBigInt(ParseBigIntError { kind: InvalidDigit }))");
    // assert_starts_with!(output, "ParseAmountError { kind: ParseAmountError, source: ParseBigInt(ParseBigIntError { kind: InvalidDigit })");
    assert_starts_with!(output, "ParseAmountError { kind: IncorrectAmount, source: ParseBigInt(ParseBigIntError { kind: InvalidDigit })");
    assert_contains!(output, "backtrace:");

    if is_anyhow_backtrace_enabled() {
        assert_contains!(output, "amount_test::fn_test_parse_amount_201\n             at ./tests/amount_test.rs:");
        assert_contains!(output, "amount_test::fn_test_parse_amount_202\n             at ./tests/amount_test.rs:");
        assert_contains!(output, "amount_test::fn_test_parse_amount_203\n             at ./tests/amount_test.rs:");
        assert_contains!(output, "amount_test::test_std_error\n             at ./tests/amount_test.rs:");
        // it is risky/dependant
        // assert_contains!(output, "amount_test::test_std_error::{{closure}}\n             at ./tests/amount_test.rs:");
    }

    // println!("\n--------------------------------------------------------\n");
    // println!("err: {err}");
    // let backtrace = match err {
    //     ParseAmountError::NoCurrencyError { backtrace } => { backtrace }
    //     ParseAmountError::ParseCurrencyError { source, backtrace } => {backtrace}
    //     ParseAmountError::ParseAmountError { source, backtrace } => { backtrace}
    // };
    // println!("my stacktrace: {}", backtrace);
    //
    // println!("\n----------------------------------------------\n");
    // println!("my stacktrace as Debug");
    // println!("my stacktrace backtrace_status: {:?}", backtrace.backtrace_status());
    // println!("my stacktrace backtrace: {}", backtrace.backtrace());
    //
    // println!("\n----------------------------------------------\n");
    // println!("my stacktrace as Display");
    // println!("my stacktrace backtrace_status: {:?}", backtrace.backtrace_status());
    // println!("my stacktrace backtrace: {:?}", backtrace.backtrace());
}


#[test]
fn test_parse_amount_error() {
    use project01::entities::amount::parse::*;

    let err = ParseAmountError::new(ErrorKind::IncorrectAmount);
    println!("err: {:?}", err)
}
