// Seems tests are not launched automatically from' macro' subproject (at least under Idea).
// As quick fix it is included there to be picked up.
//
// include!("./../error_macro/tests/macro1_test.rs");

// #[test]
// fn to_have_idea_run_tests_menu_on_this_file() { assert_eq!(1, 1); }

// -------------------------------------------------------------------------------------------------
//                     Error without source - simple variant.
// -------------------------------------------------------------------------------------------------

use assertables::{ assert_contains, assert_contains_as_result, };
use mvv_common::entity::amount::parse::AmountFormatError;
use mvv_common::entity::currency::{ CurrencyFormatError, parse::ErrorKind as CurErrorKind };
// use mvv_common::backtrace::{BacktraceCopyProvider}; // , NewBacktracePolicy};
use mvv_common::backtrace::{ BacktraceSource };
use mvv_common::test::{ TestDisplayStringOps, TestOptionDisplayStringOps };

/*
lazy_static::lazy_static! {
    static ref expected_default_bt_capture_part: &'static str =
        if is_anyhow_backtrace_enabled() { "macro1_test.rs" }
        else { "disabled" };
}
*/

fn expected_sys_default_bt_capture_part() -> &'static str {
    let contains_bt_by_default = std::backtrace::Backtrace::capture().to_string()
        .contains("macro1_test.rs");
    if contains_bt_by_default { "macro1_test.rs" }
    else { "disabled" }
}

#[test]
fn test_currency_format_error_new() {
    let err = CurrencyFormatError::new(CurErrorKind::NoCurrency);
    assert_contains!(err.backtrace.to_test_string(), "macro1_test.rs"); //  "\ncapture");
    assert_eq!(err.kind, CurErrorKind::NoCurrency);
    assert_contains!(err.backtrace_ref().to_test_string(), "macro1_test.rs"); //  "\ncapture");
}

#[test]
fn test_currency_format_error_with_backtrace() {
    use mvv_common::entity::currency::parse::{CurrencyFormatError, ErrorKind };

    let err = CurrencyFormatError::with_backtrace(ErrorKind::IncorrectCurrencyFormat); //); // , NewBacktracePolicy::Default);
    assert_contains!(err.backtrace.to_test_string(), "macro1_test.rs"); //  "\ncapture");
    assert_eq!(err.kind, ErrorKind::IncorrectCurrencyFormat);

    let err = CurrencyFormatError::with_backtrace(ErrorKind::NoCurrency); // , NewBacktracePolicy::Capture);
    assert_contains!(err.backtrace.to_test_string(), expected_sys_default_bt_capture_part()); //  "\ncapture");
    assert_contains!(err.backtrace_ref().to_test_string(), expected_sys_default_bt_capture_part()); //  , "macro1_test.rs"); //  "\ncapture");

    // let err = CurrencyFormatError::with_backtrace(ErrorKind::NoCurrency); // , NewBacktracePolicy::NoBacktrace);
    // assert_eq!(err.backtrace.to_test_string(), "Backtrace disabled");
    // assert_eq!(err.backtrace_ref().to_test_string(), "Backtrace disabled");

    let err = CurrencyFormatError::with_backtrace(ErrorKind::NoCurrency); // , NewBacktracePolicy::ForceCapture);
    assert_contains!(err.backtrace.to_test_string(), "macro1_test.rs"); //  "\nforce_capture");
    assert_contains!(err.backtrace_ref().to_test_string(), "macro1_test.rs"); //  "\nforce_capture");
}

#[test]
fn test_currency_format_error_other() {
    use anyhow::__private::kind::TraitKind;
    use thiserror::__private::AsDynError;
    use core::any::Any;
    use std::error::Error;
    use mvv_common::entity::currency::parse::{CurrencyFormatError, ErrorKind };

    let err = CurrencyFormatError::with_backtrace(ErrorKind::NoCurrency); // , NewBacktracePolicy::Default);

    // ??? What is it?
    let anyhow_kind = err.anyhow_kind();
    // anyhow::kind is private
    // let anyhow_kind: anyhow::kind::Trait = err.anyhow_kind();
    // let anyhow_kind: &dyn core::any::Any = &err.anyhow_kind();
    println!("anyhow_kind: {:?}", anyhow_kind.type_id());

    let std_err_src: Option<&dyn Error> = err.source();
    assert!(std_err_src.is_none());

    let std_err: &dyn Error = err.as_dyn_error();
    assert!(std_err.is::<CurrencyFormatError>());
}


// -------------------------------------------------------------------------------------------------
//                     Error without source - simple variant.
// -------------------------------------------------------------------------------------------------
#[test]
fn test_amount_format_error_new() {
    use mvv_common::entity::amount::parse::{ ErrorKind };

    let err = AmountFormatError::new(ErrorKind::NoCurrency);
    assert_contains!(err.backtrace.to_test_string(), "macro1_test.rs"); // "\ncapture");
    assert_eq!(err.kind, ErrorKind::NoCurrency);
    assert_contains!(err.backtrace_ref().to_test_string(), "macro1_test.rs"); // "\ncapture");
}


#[test]
fn test_amount_format_error_with_backtrace() {
    use mvv_common::entity::amount::parse::{ ErrorKind };

    let err = AmountFormatError::with_backtrace(ErrorKind::IncorrectCurrency); // , NewBacktracePolicy::Default);
    assert_contains!(err.backtrace.to_test_string(),  "macro1_test.rs"); // expected_default_bt_capture_part());
    assert_eq!(err.kind, ErrorKind::IncorrectCurrency);

    let err = AmountFormatError::with_backtrace(ErrorKind::NoCurrency); // , NewBacktracePolicy::Capture);
    assert_contains!(err.backtrace.to_test_string(), expected_sys_default_bt_capture_part()); //  "macro1_test.rs"); // "\ncapture");
    assert_contains!(err.backtrace_ref().to_test_string(), expected_sys_default_bt_capture_part()); // , "macro1_test.rs"); // "\ncapture");

    // let err = AmountFormatError::with_backtrace(ErrorKind::IncorrectAmount); // , NewBacktracePolicy::NoBacktrace);
    // assert_eq!(err.backtrace.to_test_string(), "Backtrace disabled");
    // assert_eq!(err.backtrace_ref().to_test_string(), "Backtrace disabled");

    let err = AmountFormatError::with_backtrace(ErrorKind::NoCurrency); // , NewBacktracePolicy::ForceCapture);
    assert_contains!(err.backtrace.to_test_string(), "macro1_test.rs"); // "\nforce_capture");
    assert_contains!(err.backtrace_ref().to_test_string(), "macro1_test.rs"); // "\nforce_capture");
}


#[test]
fn test_amount_format_error_with_source() {
    use mvv_common::entity::currency::{self, parse::CurrencyFormatError };
    use mvv_common::entity::amount::parse::{ErrorKind, ErrorSource };

    let err = AmountFormatError::with_source(ErrorKind::NoCurrency, ErrorSource::NoSource);
    assert_eq!(err.kind, ErrorKind::NoCurrency);
    assert_contains!(err.backtrace.to_test_string(), "macro1_test.rs"); // "\ncapture");
    assert_contains!(err.backtrace_ref().to_test_string(), "macro1_test.rs"); //  "\ncapture");

    let err = AmountFormatError::with_source(ErrorKind::NoCurrency, ErrorSource::CurrencyFormatError(
        CurrencyFormatError::with_backtrace(currency::parse::ErrorKind::NoCurrency))); // , NewBacktracePolicy::ForceCapture)));
    assert_eq!(err.kind, ErrorKind::NoCurrency);
    assert_contains!(err.backtrace.to_test_string(), "macro1_test.rs"); // "\nforce_capture");
    assert_contains!(err.backtrace_ref().to_test_string(), "macro1_test.rs"); // "\nforce_capture");

    let err = AmountFormatError::with_from(ErrorKind::NoCurrency,
                                           CurrencyFormatError::with_backtrace(currency::parse::ErrorKind::NoCurrency)); // , NewBacktracePolicy::ForceCapture));
    assert_eq!(err.kind, ErrorKind::NoCurrency);
    assert_contains!(err.backtrace.to_test_string(), "macro1_test.rs"); // "\nforce_capture");
    assert_contains!(err.backtrace_ref().to_test_string(), "macro1_test.rs"); // "\nforce_capture");

    let err = AmountFormatError::with_source(ErrorKind::IncorrectCurrency, ErrorSource::CurrencyFormatError(
        CurrencyFormatError::with_backtrace(currency::parse::ErrorKind::NoCurrency))); // , NewBacktracePolicy::ForceCapture)));
    assert_eq!(err.kind, ErrorKind::IncorrectCurrency);
    assert_contains!(err.backtrace.to_test_string(), "macro1_test.rs"); // "\nforce_capture");
    assert_contains!(err.backtrace_ref().to_test_string(), "macro1_test.rs"); // "\nforce_capture");

    let err = AmountFormatError::with_from(ErrorKind::IncorrectCurrency,
                                           CurrencyFormatError::with_backtrace(currency::parse::ErrorKind::NoCurrency)); // , NewBacktracePolicy::ForceCapture));
    assert_eq!(err.kind, ErrorKind::IncorrectCurrency);
    assert_contains!(err.backtrace.to_test_string(), "macro1_test.rs"); // "\nforce_capture");
    assert_contains!(err.backtrace_ref().to_test_string(), "macro1_test.rs"); // "\nforce_capture");

    let err = AmountFormatError::with_source(ErrorKind::IncorrectAmount, ErrorSource::ParseBigDecimalError(
        bigdecimal::ParseBigDecimalError::Other("some decimal error".to_test_string())));
    assert_eq!(err.kind, ErrorKind::IncorrectAmount);
    assert_contains!(err.backtrace.to_test_string(), "macro1_test.rs"); // "\ncapture");
    assert_contains!(err.backtrace_ref().to_test_string(), "macro1_test.rs"); // "\ncapture");

    let err = AmountFormatError::with_from(ErrorKind::IncorrectAmount,
                                           bigdecimal::ParseBigDecimalError::Other("some decimal error".to_test_string()));
    assert_eq!(err.kind, ErrorKind::IncorrectAmount);
    assert_contains!(err.backtrace.to_test_string(), "macro1_test.rs"); // "\ncapture");
    assert_contains!(err.backtrace_ref().to_test_string(), "macro1_test.rs"); // "\ncapture");
}


#[test]
fn test_amount_format_error_src() {
    use mvv_common::entity::currency::parse::{CurrencyFormatError, ErrorKind };
    use mvv_common::entity::amount::parse::{ ErrorSource };

    let err = CurrencyFormatError::with_backtrace(ErrorKind::NoCurrency); // , NewBacktracePolicy::Default);
    assert_contains!(err.backtrace_ref().to_test_string(), "macro1_test.rs"); // "\ncapture");

    let err_src: ErrorSource = err.into();
    match err_src {
        ErrorSource::CurrencyFormatError(_) => { }
        _ => { assert!(false, "Unexpected amount error source type.") }
    }
}

#[allow(dead_code)]
fn fn_anyhow_01() -> Result<i32, anyhow::Error> {
    std::fs::read_to_string("not-existent-file.txt").map(|_| 123) ?;
    Ok(124)
}
#[allow(dead_code)]
fn fn_anyhow_02() -> Result<i32, anyhow::Error> { fn_anyhow_01() }
