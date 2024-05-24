
mod util {

    #[allow(unused_imports)]
    pub use backtrace::BacktraceInfo;

    pub mod backtrace {
        #[derive(Clone)]
        pub struct BacktraceInfo {
            backtrace: String,
        }

        impl std::fmt::Display for BacktraceInfo {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "\n{}", self.backtrace)
            }
        }
        impl std::fmt::Debug for BacktraceInfo {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "\n{}", self.backtrace)
            }
        }

        impl BacktraceInfo {
            pub fn new() -> Self { Self::capture() }
            pub fn from_str(str: &str) -> Self { Self::from_string(str.to_string()) }
            pub fn from_string(string: String) -> Self { BacktraceInfo { backtrace: string } }
            pub fn copy_from(_backtrace: std::backtrace::Backtrace) -> Self { Self::capture() }
            pub fn new_by_policy(backtrace_policy: NewBacktracePolicy) -> Self {
                use NewBacktracePolicy::*;
                match backtrace_policy {
                    Default | Capture => { Self::capture() }
                    NoBacktrace       => { Self::empty() }
                    ForceCapture      => { Self::force_capture() }
                }
            }
            pub fn capture() -> Self { BacktraceInfo { backtrace: "capture".to_string() } }
            pub fn force_capture() -> Self { BacktraceInfo { backtrace: "force_capture".to_string() } }
            pub fn empty() -> Self { BacktraceInfo { backtrace: "empty".to_string() } }
            pub fn inherit_from<BP: BacktraceCopyProvider>(source: &BP) -> Self {
                Self::inherit_with_policy(source, InheritBacktracePolicy::Default)
            }
            pub fn inherit_with_policy<BP: BacktraceCopyProvider>(source: &BP, backtrace_policy: InheritBacktracePolicy) -> Self {
                Self::reuse(source.provide_backtrace(), backtrace_policy)
            }
            fn reuse(source_bt: BacktraceInfo, backtrace_policy: InheritBacktracePolicy) -> Self {
                use InheritBacktracePolicy::*;
                match backtrace_policy {
                    Inherit                     => { source_bt }
                    Default | InheritOrCapture  => { if source_bt.is_captured() { source_bt } else { Self::capture() } }
                    InheritOrForceCapture       => { if source_bt.is_captured() { source_bt } else { Self::force_capture() } }
                }
            }
            pub fn is_captured(&self) -> bool { self.backtrace != "empty" && self.backtrace != "disabled" }
        }

        pub enum NewBacktracePolicy {
            Default,
            NoBacktrace,
            Capture,
            ForceCapture,
        }
        // should be used together with other/source/from Error
        pub enum InheritBacktracePolicy {
            Default,
            Inherit,
            InheritOrCapture,
            InheritOrForceCapture,
        }

        pub trait BacktraceCopyProvider {
            // Using 'provide' name causes warning 'unstable_name_collision'
            fn provide_backtrace(&self) -> BacktraceInfo;
            fn contains_self_or_child_captured_backtrace(&self) -> bool {
                self.provide_backtrace().is_captured()
            }
        }
        #[allow(dead_code)]
        pub trait BacktraceBorrowedProvider { // or better Moved???
            fn provide_backtrace(&self) -> BacktraceInfo;
        }


        impl BacktraceCopyProvider for anyhow::Error {
            fn provide_backtrace(&self) -> BacktraceInfo {
                BacktraceInfo::from_string(self.backtrace().to_string())
            }
        }

        fn std_backtrace_of_std_err<'a>(_err: & 'a dyn std::error::Error) -> Option<& 'a std::backtrace::Backtrace> {
            None
        }

        impl BacktraceCopyProvider for Box<dyn std::error::Error> {
            fn provide_backtrace(&self) -> BacktraceInfo { Some(self.as_ref()).provide_backtrace() }
            fn contains_self_or_child_captured_backtrace(&self) -> bool {
                Some(self.as_ref()).contains_self_or_child_captured_backtrace()
            }
        }

        impl<'a> BacktraceCopyProvider for Option<& 'a dyn std::error::Error> {
            fn provide_backtrace(&self) -> BacktraceInfo {
                let std_err_opt = self.and_then(|err| std_backtrace_of_std_err(err));
                std_err_opt.map(|bt| BacktraceInfo::from_string(bt.to_string())).unwrap_or(BacktraceInfo::empty())
            }

            fn contains_self_or_child_captured_backtrace(&self) -> bool {
                let std_err_opt = self.and_then(|err| std_backtrace_of_std_err(err));
                std_err_opt.map(|bt| bt.status() == std::backtrace::BacktraceStatus::Captured).unwrap_or(false)
            }
        }

        impl BacktraceCopyProvider for String {
            fn provide_backtrace(&self) -> BacktraceInfo { BacktraceInfo::empty() }
            fn contains_self_or_child_captured_backtrace(&self) -> bool { false }
        }

    }
}


pub mod parse_currency {
    // use static_error_macro::MyStaticStructError;
    use crate::util::backtrace::BacktraceInfo;

    // #[derive(Debug, PartialEq, Copy, Clone)]
    #[derive(Debug, thiserror::Error)]
    #[derive(Copy, Clone)]
    #[derive(PartialOrd, PartialEq)]
    pub enum ErrorKind {
        #[error("no currency")]
        NoCurrency,
        #[error("Incorrect currency format")]
        IncorrectCurrencyFormat,
    }

    // #[derive(Debug, PartialEq, Copy, Clone)]
    #[derive(thiserror::Error)]
    #[derive(static_error_macro::MyStaticStructError)]
    pub struct CurrencyFormatError {
        pub kind: ErrorKind,
        // #[source]
        // pub source: ErrorSource,
        pub backtrace: BacktraceInfo,
    }

    // #[derive(thiserror::Error)]
    // pub enum ErrorSource {
    //     #[error("No source")]
    //     NoSource,
    // }

    /*
    impl CurrencyFormatError {
        // It can be generated by macro
        pub fn new(kind: ErrorKind) -> Self {
            Self { kind, backtrace: BacktraceInfo::new() }
        }
        // It can be generated by macro
        pub fn with_backtrace(kind: ErrorKind, backtrace_policy: NewBacktracePolicy) -> Self {
            Self { kind, backtrace: BacktraceInfo::new_by_policy(backtrace_policy) }
        }
    }

    // It can be generated by macro
    impl BacktraceCopyProvider for CurrencyFormatError {
        fn provide_backtrace(&self) -> BacktraceInfo { self.backtrace.clone() }
    }

    // It can be generated by macro
    impl fmt::Display for CurrencyFormatError {
        fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
            write!(f, "CurrencyFormatError  {}", self.kind)
        }
    }
    */
}



pub mod parse_amount {
    use bigdecimal::ParseBigDecimalError;
    use crate::util::backtrace::{ BacktraceCopyProvider, BacktraceInfo };
    // use crate::entities::currency::parse_currency::CurrencyFormatError;
    use crate::parse_currency::CurrencyFormatError;

    #[derive(Debug, thiserror::Error)]
    #[derive(Copy, Clone)]
    #[derive(PartialOrd, PartialEq)]
    pub enum ErrorKind {
        #[error("No currency in amount")]
        NoCurrency,
        #[error("Incorrect currency format")]
        IncorrectCurrency,
        #[error("Incorrect amount format")]
        IncorrectAmount,
    }

    #[derive(thiserror::Error)]
    #[derive(static_error_macro::MyStaticStructError)]
    pub struct ParseAmountError {
        pub kind: ErrorKind,
        #[source]
        // #[from]
        pub source: ErrorSource,
        pub backtrace: BacktraceInfo,
    }

    // It can be generated by macro
    #[derive(thiserror::Error)] // TODO: remove
    #[derive(static_error_macro::MyStaticStructErrorSource)]
    // Full type or short type can be used: ParseAmountError/crate::entities::amount::parse_amount::ParseAmountError
    // #[struct_error_type(ParseAmountError)]
    #[struct_error_type(ParseAmountError)]
    #[do_not_generate_std_error]
    pub enum ErrorSource {
        // #[error("No source")]
        NoSource,
        // #[error("Currency format error")]
        // #[static_error_macro::StaticStructErrorType(ParseAmountError)]
        // #[from_error_kind(IncorrectCurrency)]
        CurrencyFormatError(CurrencyFormatError),
        // for testing
        // CurrencyFormatError22(crate::entities::currency::parse_currency::CurrencyFormatError),
        // #[error("Decimal format error")]
        #[from_error_kind(IncorrectAmount)]
        #[no_source_backtrace]
        ParseBigDecimalError(ParseBigDecimalError),

        // #[no_source_backtrace] // TODO: make it automatic
        // SomeWithoutSource,

        // With duplicated types
        // #[error("Some1FromString")]
        #[no_source_backtrace]
        // #[from_error_kind(IncorrectAmount)] // temp. to test proper 'duplicates' error
        Some1FromString(String),
        // #[error("Some2FromString")]
        #[no_source_backtrace]
        // #[from_error_kind(IncorrectAmount)] // temp. to test proper 'duplicates' error
        Some2FromString(String),
        // #[error("Some1FromInt")]
        #[no_source_backtrace]
        Some1FromInt(i32),
        // #[error("Some2FromInt")]
        #[no_source_backtrace]
        Some2FromInt(i32),

        // #[error("SomeAnyHowError")]
        SomeAnyHowError(anyhow::Error),

        // #[error("SomeStdError")]
        StdErrorError(Box<dyn std::error::Error>),
    }

    /*
    impl ParseAmountError {

        // error[E0658]: inherent associated types are unstable
        // see issue #8995 <https://github.com/rust-lang/rust/issues/8995> for more information
        //
        // type ErrorKind = ParseAmountErrorKind;

        // It can be generated by macro
        pub fn new(kind: ErrorKind) -> ParseAmountError {
            ParseAmountError { kind, source: ErrorSource::NoSource, backtrace: BacktraceInfo::new() }
        }
        // It can be generated by macro
        pub fn with_backtrace(kind: ErrorKind, backtrace_policy: NewBacktracePolicy) -> ParseAmountError {
            ParseAmountError { kind, source: ErrorSource::NoSource, backtrace: BacktraceInfo::new_by_policy(backtrace_policy) }
        }
        // It can be generated by macro
        pub fn with_source(kind: ErrorKind, source: ErrorSource) -> ParseAmountError {
            ParseAmountError { kind, backtrace: BacktraceInfo::inherit_from(&source), source }
        }
        // It can be generated by macro
        pub fn with_from<ES: Into<ErrorSource>>(kind: ErrorKind, source: ES) -> ParseAmountError {
            let src = source.into();
            ParseAmountError { kind, backtrace: BacktraceInfo::inherit_from(&src), source: src }
        }
    }

    // It can be generated by macro
    impl From<CurrencyFormatError> for ParseAmountError {
        fn from(error: CurrencyFormatError) -> Self { ParseAmountError::with_from(ErrorKind::IncorrectCurrency, error) }
    }
    // It can be generated by macro
    impl From<ParseBigDecimalError> for ParseAmountError {
        fn from(error: ParseBigDecimalError) -> Self { ParseAmountError::with_from(ErrorKind::IncorrectAmount, error) }
    }

    // It can be generated by macro
    impl Into<ErrorSource> for CurrencyFormatError {
        fn into(self) -> ErrorSource { ErrorSource::CurrencyFormatError22(self) }
    }
    // It can be generated by macro
    impl Into<ErrorSource> for ParseBigDecimalError {
        fn into(self) -> ErrorSource { ErrorSource::ParseBigDecimalError(self) }
    }

    // It can be generated by macro
    impl BacktraceCopyProvider for ErrorSource {
        fn provide_backtrace(&self) -> BacktraceInfo {
            match self {
                ErrorSource::NoSource => { BacktraceInfo::empty() }
                ErrorSource::ParseBigDecimalError(_)  => { BacktraceInfo::empty() }
                ErrorSource::CurrencyFormatError22(src) => { src.provide_backtrace() }
            }
        }
    }

    // It can be generated by macro
    impl core::fmt::Debug for ErrorSource {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
            use ErrorSource::*;
            match self {
                NoSource                      => { write!(f, "No source") }
                CurrencyFormatError22(ref src)  => { write!(f, "{:?}", src) }
                ParseBigDecimalError(ref src) => { write!(f, "{:?}", src) }
            }
        }
    }

    // It can be generated by macro
    impl core::fmt::Display for ParseAmountError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{}", self.kind)
        }
    }

    // impl BacktraceCopyProvider for ParseBigDecimalError {
    //     fn provide_backtrace(&self) -> BacktraceInfo { BacktraceInfo::empty() }
    // }
    */
}

// -------------------------------------------------------------------------------------------------
//                     Error without source - simple variant.
// -------------------------------------------------------------------------------------------------

#[test]
fn test_currency_format_error_new() {
    use parse_currency::*;
    use crate::util::backtrace::BacktraceCopyProvider;

    let err = CurrencyFormatError::new(ErrorKind::NoCurrency);
    assert_eq!(err.backtrace.to_string(), r#"BacktraceInfo { backtrace: "capture" }"#);
    assert_eq!(err.kind, ErrorKind::NoCurrency);
    assert_eq!(err.provide_backtrace().to_string(), r#"BacktraceInfo { backtrace: "capture" }"#);
}

#[test]
fn test_currency_format_error_with_backtrace() {
    use parse_currency::*;
    use crate::util::backtrace::NewBacktracePolicy;
    use crate::util::backtrace::BacktraceCopyProvider;

    let err = CurrencyFormatError::with_backtrace(ErrorKind::IncorrectCurrencyFormat, NewBacktracePolicy::Default);
    assert_eq!(err.backtrace.to_string(), r#"BacktraceInfo { backtrace: "capture" }"#);
    assert_eq!(err.kind, ErrorKind::IncorrectCurrencyFormat);

    let err = CurrencyFormatError::with_backtrace(ErrorKind::NoCurrency, NewBacktracePolicy::Capture);
    assert_eq!(err.backtrace.to_string(), r#"BacktraceInfo { backtrace: "capture" }"#);
    assert_eq!(err.provide_backtrace().to_string(), r#"BacktraceInfo { backtrace: "capture" }"#);

    let err = CurrencyFormatError::with_backtrace(ErrorKind::NoCurrency, NewBacktracePolicy::NoBacktrace);
    assert_eq!(err.backtrace.to_string(), r#"BacktraceInfo { backtrace: "empty" }"#);
    assert_eq!(err.provide_backtrace().to_string(), r#"BacktraceInfo { backtrace: "empty" }"#);

    let err = CurrencyFormatError::with_backtrace(ErrorKind::NoCurrency, NewBacktracePolicy::ForceCapture);
    assert_eq!(err.backtrace.to_string(), r#"BacktraceInfo { backtrace: "force_capture" }"#);
    assert_eq!(err.provide_backtrace().to_string(), r#"BacktraceInfo { backtrace: "force_capture" }"#);
}

#[test]
fn test_currency_format_error_other() {
    use parse_currency::*;
    use crate::util::backtrace::NewBacktracePolicy;
    use anyhow::__private::kind::TraitKind;
    use thiserror::__private::AsDynError;
    use std::any::Any;
    use std::error::Error;

    let err = CurrencyFormatError::with_backtrace(ErrorKind::NoCurrency, NewBacktracePolicy::Default);

    // ??? What is it?
    let anyhow_kind = err.anyhow_kind();
    // anyhow::kind is private
    // let anyhow_kind: anyhow::kind::Trait = err.anyhow_kind();
    // let anyhow_kind: &dyn std::any::Any = &err.anyhow_kind();
    println!("anyhow_kind: {:?}", anyhow_kind.type_id());

    let std_err_src: Option<&dyn Error> = err.source();
    // T O D O: add support of it after appearing std::error::Error.provide() in stable build.
    assert!(std_err_src.is_none());

    let std_err: &dyn Error = err.as_dyn_error();
    assert!(std_err.is::<CurrencyFormatError>());
}



// -------------------------------------------------------------------------------------------------
//                     Error without source - simple variant.
// -------------------------------------------------------------------------------------------------


#[test]
fn test_amount_format_error_new() {
    use parse_amount::*;
    use crate::util::backtrace::BacktraceCopyProvider;

    let err = ParseAmountError::new(ErrorKind::NoCurrency);
    assert_eq!(err.backtrace.to_string(), r#"BacktraceInfo { backtrace: "capture" }"#);
    assert_eq!(err.kind, ErrorKind::NoCurrency);
    assert_eq!(err.provide_backtrace().to_string(), r#"BacktraceInfo { backtrace: "capture" }"#);
}


#[test]
fn test_amount_format_error_with_backtrace() {
    use parse_amount::*;
    use crate::util::backtrace::NewBacktracePolicy;
    use crate::util::backtrace::BacktraceCopyProvider;

    let err = ParseAmountError::with_backtrace(ErrorKind::IncorrectCurrency, NewBacktracePolicy::Default);
    assert_eq!(err.backtrace.to_string(), r#"BacktraceInfo { backtrace: "capture" }"#);
    assert_eq!(err.kind, ErrorKind::IncorrectCurrency);

    let err = ParseAmountError::with_backtrace(ErrorKind::NoCurrency, NewBacktracePolicy::Capture);
    assert_eq!(err.backtrace.to_string(), r#"BacktraceInfo { backtrace: "capture" }"#);
    assert_eq!(err.provide_backtrace().to_string(), r#"BacktraceInfo { backtrace: "capture" }"#);

    let err = ParseAmountError::with_backtrace(ErrorKind::IncorrectAmount, NewBacktracePolicy::NoBacktrace);
    assert_eq!(err.backtrace.to_string(), r#"BacktraceInfo { backtrace: "empty" }"#);
    assert_eq!(err.provide_backtrace().to_string(), r#"BacktraceInfo { backtrace: "empty" }"#);

    let err = ParseAmountError::with_backtrace(ErrorKind::NoCurrency, NewBacktracePolicy::ForceCapture);
    assert_eq!(err.backtrace.to_string(), r#"BacktraceInfo { backtrace: "force_capture" }"#);
    assert_eq!(err.provide_backtrace().to_string(), r#"BacktraceInfo { backtrace: "force_capture" }"#);
}


#[test]
fn test_amount_format_error_with_source() {
    use parse_amount::*;
    use crate::util::backtrace::NewBacktracePolicy;
    use crate::util::backtrace::BacktraceCopyProvider;

    let err = ParseAmountError::with_source(ErrorKind::NoCurrency, ErrorSource::NoSource);
    assert_eq!(err.kind, ErrorKind::NoCurrency);
    assert_eq!(err.backtrace.to_string(), r#"BacktraceInfo { backtrace: "capture" }"#);
    assert_eq!(err.provide_backtrace().to_string(), r#"BacktraceInfo { backtrace: "capture" }"#);

    let err = ParseAmountError::with_source(ErrorKind::NoCurrency, ErrorSource::CurrencyFormatError(
        parse_currency::CurrencyFormatError::with_backtrace(parse_currency::ErrorKind::NoCurrency, NewBacktracePolicy::ForceCapture)));
    assert_eq!(err.kind, ErrorKind::NoCurrency);
    assert_eq!(err.backtrace.to_string(), r#"BacktraceInfo { backtrace: "force_capture" }"#);
    assert_eq!(err.provide_backtrace().to_string(), r#"BacktraceInfo { backtrace: "force_capture" }"#);

    let err = ParseAmountError::with_from(ErrorKind::NoCurrency,
        parse_currency::CurrencyFormatError::with_backtrace(parse_currency::ErrorKind::NoCurrency, NewBacktracePolicy::ForceCapture));
    assert_eq!(err.kind, ErrorKind::NoCurrency);
    assert_eq!(err.backtrace.to_string(), r#"BacktraceInfo { backtrace: "force_capture" }"#);
    assert_eq!(err.provide_backtrace().to_string(), r#"BacktraceInfo { backtrace: "force_capture" }"#);

    let err = ParseAmountError::with_source(ErrorKind::IncorrectCurrency, ErrorSource::CurrencyFormatError(
        parse_currency::CurrencyFormatError::with_backtrace(parse_currency::ErrorKind::NoCurrency, NewBacktracePolicy::ForceCapture)));
    assert_eq!(err.kind, ErrorKind::IncorrectCurrency);
    assert_eq!(err.backtrace.to_string(), r#"BacktraceInfo { backtrace: "force_capture" }"#);
    assert_eq!(err.provide_backtrace().to_string(), r#"BacktraceInfo { backtrace: "force_capture" }"#);

    let err = ParseAmountError::with_from(ErrorKind::IncorrectCurrency,
        parse_currency::CurrencyFormatError::with_backtrace(parse_currency::ErrorKind::NoCurrency, NewBacktracePolicy::ForceCapture));
    assert_eq!(err.kind, ErrorKind::IncorrectCurrency);
    assert_eq!(err.backtrace.to_string(), r#"BacktraceInfo { backtrace: "force_capture" }"#);
    assert_eq!(err.provide_backtrace().to_string(), r#"BacktraceInfo { backtrace: "force_capture" }"#);

    let err = ParseAmountError::with_source(ErrorKind::IncorrectAmount, ErrorSource::ParseBigDecimalError(
        bigdecimal::ParseBigDecimalError::Other("some decimal error".to_string())));
    assert_eq!(err.kind, ErrorKind::IncorrectAmount);
    assert_eq!(err.backtrace.to_string(), r#"BacktraceInfo { backtrace: "capture" }"#);
    assert_eq!(err.provide_backtrace().to_string(), r#"BacktraceInfo { backtrace: "capture" }"#);

    let err = ParseAmountError::with_from(ErrorKind::IncorrectAmount,
        bigdecimal::ParseBigDecimalError::Other("some decimal error".to_string()));
    assert_eq!(err.kind, ErrorKind::IncorrectAmount);
    assert_eq!(err.backtrace.to_string(), r#"BacktraceInfo { backtrace: "capture" }"#);
    assert_eq!(err.provide_backtrace().to_string(), r#"BacktraceInfo { backtrace: "capture" }"#);
}


#[test]
fn test_amount_format_error_with_source_666() {
    use parse_amount::*;
    // use crate::util::backtrace::NewBacktracePolicy;
    use crate::util::backtrace::BacktraceCopyProvider;

    // let err = ParseAmountError::with_source(ErrorKind::NoCurrency, ErrorSource::SomeWithoutSource); // TODO: use it
    let err = ParseAmountError::with_source(ErrorKind::NoCurrency, ErrorSource::Some1FromInt(666));
    assert_eq!(err.kind, ErrorKind::NoCurrency);
    assert_eq!(err.backtrace.to_string(), "\ncapture");
    assert_eq!(err.provide_backtrace().to_string(), "\ncapture");
}


#[test]
fn test_amount_format_error_src() {
    use parse_amount::ErrorSource;
    use parse_currency::{ CurrencyFormatError, ErrorKind };
    use crate::util::backtrace::NewBacktracePolicy;
    use crate::util::backtrace::BacktraceCopyProvider;

    let err = CurrencyFormatError::with_backtrace(ErrorKind::NoCurrency, NewBacktracePolicy::Default);
    assert_eq!(err.provide_backtrace().to_string(), r#"BacktraceInfo { backtrace: "capture" }"#);

    let err_src: ErrorSource = err.into();
    // assert_eq!(err_src.type_id(), ErrorSource::CurrencyFormatError())
    match err_src {
        ErrorSource::CurrencyFormatError(_) => { }
        _ => { assert!(false, "Unexpected amount error source type.") }
    }
}

fn fn_anyhow_01() -> Result<i32, anyhow::Error> {
    std::fs::read_to_string("not-existent-file.txt").map(|_| 123) ?;
    Ok(124)
}
fn fn_anyhow_02() -> Result<i32, anyhow::Error> { fn_anyhow_01() }

#[test]
fn test_amount_format_error_src_from_anyhow() {
    use parse_amount::*;
    use std::fmt::Write;

    let err_res = fn_anyhow_02();
    let res = err_res.map_err(|anyhow_err| ParseAmountError::with_source(ErrorKind::IncorrectAmount, ErrorSource::SomeAnyHowError(anyhow_err)));

    let amount_err = res.err().unwrap();

    println!("amount_err from anyhow: {:?}", amount_err);
    println!("\n-------------------------------------------\n");

    let mut amount_err_as_str_with_backtrace = String::new();
    write!(amount_err_as_str_with_backtrace, "{:?}", amount_err).unwrap();

    println!("{}", amount_err_as_str_with_backtrace);

    assert!(amount_err_as_str_with_backtrace.contains("fn_anyhow_01"));
    assert!(amount_err_as_str_with_backtrace.contains("fn_anyhow_02"));

}
