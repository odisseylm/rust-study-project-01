use std::str::FromStr;
use bigdecimal::BigDecimal;
use crate::entities::amount::Amount;
use crate::entities::currency::Currency;
use crate::util::BacktraceInfo;


#[allow(dead_code)] // it is used by private test
fn parse_amount_01(s: &str) -> Result<Amount, parse_amount_old::ParseAmountError> {
    use parse_amount_old::{ ParseAmountError, ErrorKind };

    let s = s.trim();

    let last_space_bytes_offset = s.rfind(|ch: char|{ ch.is_ascii_whitespace() })
        .ok_or_else(|| ParseAmountError::new(ErrorKind::NoCurrency)) ?;

    let (str_amount, str_cur) = s.split_at(last_space_bytes_offset);

    let currency = Currency::from_str(str_cur.trim_start()) ?;
    let amount = BigDecimal::from_str(str_amount.trim_end()) ?;

    Ok(Amount::new(amount, currency))

    /*
    let s = s.trim();
    let last_space_bytes_offset_opt: Option<usize> = s.rfind(|ch: char|{ ch.is_ascii_whitespace() });

    match last_space_bytes_offset_opt {
        None => { Err(ParseAmountError::NoCurrencyError) }
        Some(last_space_bytes_offset) => {
            let (str_amount, str_cur) = s.split_at(last_space_bytes_offset);
            let currency_res = Currency::from_str(str_cur.trim_start());

            match currency_res {
                Err(cur_parse_err) => { Err(ParseAmountError::ParseCurrencyError(cur_parse_err)) }
                Ok(currency) => {
                    let amount_res = BigDecimal::from_str(str_amount.trim_end());

                    match amount_res {
                        Err(amount_parse_err) => { Err(ParseAmountError::ParseAmountError(amount_parse_err)) }
                        Ok(amount) => { Ok(Amount::new(amount, currency)) }
                    }
                }
            }
        }
    }
    */
}


#[allow(dead_code)] // it is used by private test
fn parse_amount_02(s: &str) -> Result<Amount, parse_amount_old::ParseAmountError> {
    use parse_amount_old::ErrorSource::*;
    use parse_amount_old::{ ParseAmountError, ErrorKind };

    let s = s.trim();

    let last_space_bytes_offset = s.rfind(|ch: char|{ ch.is_ascii_whitespace() })
        .ok_or_else(|| ParseAmountError::new(ErrorKind::NoCurrency)) ?;

    let (str_amount, str_cur) = s.split_at(last_space_bytes_offset);

    let currency = Currency::from_str(str_cur.trim_start())
        .map_err(|er| ParseAmountError::with_source(ErrorKind::IncorrectCurrency, CurrencyFormatError(er))) ?;

    let amount = BigDecimal::from_str(str_amount.trim_end())
        .map_err(|er| ParseAmountError { kind: ErrorKind::IncorrectAmount,
            source: ParseBigDecimalError(er), backtrace: BacktraceInfo::new() }) ?;

    Ok(Amount::new(amount, currency))
}


#[allow(dead_code)] // it is used by private test
fn parse_amount_03(s: &str) -> Result<Amount, parse_amount_old::ParseAmountError> {
    use parse_amount_old::{ ParseAmountError, ErrorKind };

    let s = s.trim();

    let last_space_bytes_offset = s.rfind(|ch: char|{ ch.is_ascii_whitespace() })
        .ok_or_else(|| ParseAmountError::new(ErrorKind::NoCurrency)) ?;

    let (str_amount, str_cur) = s.split_at(last_space_bytes_offset);

    let currency = Currency::from_str(str_cur.trim_start())
        .map_err(|er| ParseAmountError::with_from(ErrorKind::IncorrectCurrency, er)) ?;

    let amount = BigDecimal::from_str(str_amount.trim_end())
        .map_err(|er| ParseAmountError::with_from(ErrorKind::IncorrectAmount, er)) ?;

    Ok(Amount::new(amount, currency))
}


// -------------------------------------------------------------------------------------------------
//                                        Error
// -------------------------------------------------------------------------------------------------

#[allow(dead_code)]
struct S1 { _x: i32 }

#[allow(dead_code)]
trait ST1 {
    fn get_x(&self) -> i32;
}
impl ST1 for S1 {
    fn get_x(&self) -> i32 { self._x }
}

/*
fn aa_01() {
    let ptr0: Box<S1> = Box::new(S1{_x:666});
    let ptr1: Box<dyn ST1> = Box::new(S1{_x:666});

    let s1 = S1{_x:666};
    let p_s1: &S1 = &s1;
    let p_s1: & dyn ST1 = &s1;
    // let p_s1 = & dyn s1;

    let anyhow_err: anyhow::Error = t o d o!();
    parse_amount_old::ParseAmountError::with_source(parse_amount_old::ErrorKind::IncorrectAmount, ErrorSource::SomeAnyHowError(anyhow_err));

    let anyhow_err: anyhow::Error = t o d o!();
    parse_amount_old::ParseAmountError::with_from(parse_amount_old::ErrorKind::IncorrectAmount, anyhow_err);
}
*/

// rust does not support nested structs/types/so on.
// As workaround, I decided to use sub-namespace.
//
pub mod parse_amount_old {
    use bigdecimal::ParseBigDecimalError;
    use crate::util::BacktraceInfo;
    use crate::entities::currency::parse_currency::CurrencyFormatError;

    //noinspection DuplicatedCode
    #[derive(Debug, thiserror::Error)]
    #[derive(Copy, Clone)]
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

    //noinspection DuplicatedCode
    // It can be generated by macro
    #[derive(static_error_macro::MyStaticStructErrorSource)]
    // Full type or short type can be used: ParseAmountError/crate::entities::amount::parse_amount::ParseAmountError
    #[struct_error_type(ParseAmountError)]
    // #[do_not_generate_std_error]
    pub enum ErrorSource {
        //#[error("No source")]
        NoSource,
        //#[error("Currency format error")]
        // #[static_error_macro::StaticStructErrorType(ParseAmountError)]
        #[from_error_kind(IncorrectCurrency)]
        CurrencyFormatError(CurrencyFormatError),
        // for testing
        // CurrencyFormatError22(crate::entities::currency::parse_currency::CurrencyFormatError),
        //#[error("Decimal format error")]
        #[from_error_kind(IncorrectAmount)]
        #[no_source_backtrace]
        ParseBigDecimalError(ParseBigDecimalError),

        // just for test
        SomeInt64(i64),

        SomeWithoutArg,

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
        StdError(Box<dyn std::error::Error>),
    }

    /*
    impl std::error::Error for ErrorSource {
        fn source(& self) -> Option<& (dyn std::error::Error + 'static)> {
            match self {
                ErrorSource::NoSource => None,
                ErrorSource::CurrencyFormatError(ref src) => Some(src),
                ErrorSource::ParseBigDecimalError(ref src) => Some(src),
                ErrorSource::SomeInt64(_) => None,
                ErrorSource::SomeWithoutArg => None,
                ErrorSource::Some1FromString(_) => None,
                ErrorSource::Some2FromString(_) => None,
                ErrorSource::Some1FromInt(_) => None,
                ErrorSource::Some2FromInt(_) => None,
                ErrorSource::SomeAnyHowError(ref src) => {
                    // let p_s0: & dyn std::error::Error = src.as_ref();
                    // Some(p_s0)
                    Some(src.as_ref())
                },
                ErrorSource::StdError(ref src) => Some(src.as_ref()),
                _ => None,
            }
        }
        #[allow(deprecated)]
        fn description(&self) -> &str {
            match self {
                ErrorSource::NoSource => "",
                ErrorSource::CurrencyFormatError(ref src) => src.description(),
                ErrorSource::ParseBigDecimalError(ref src) => src.description(),
                ErrorSource::SomeInt64(_) => "",
                ErrorSource::SomeWithoutArg => "",
                ErrorSource::Some1FromString(_) => "",
                ErrorSource::Some2FromString(_) => "",
                ErrorSource::Some1FromInt(_) => "",
                ErrorSource::Some2FromInt(_) => "",
                ErrorSource::SomeAnyHowError(ref src) => src.description(),
                ErrorSource::StdError(ref src) => src.description(),
            }
        }
    }
    */


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

    impl core::fmt::Display for ErrorSource {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            match self {
                // ErrorSource::NoSource                => { write!(f, "NoSource 666") }
                ErrorSource::NoSource                => { write!(f, "NoSource") }
                // Use where no source or if marked #[use_name_as_display]
                // ErrorSource::CurrencyFormatError(_)  => { write!(f, "CurrencyFormatError 666")  }
                // ErrorSource::ParseBigDecimalError(_) => { write!(f, "ParseBigDecimalError 666") }
                ErrorSource::CurrencyFormatError(src)  => { write!(f, "{}", src) }
                ErrorSource::ParseBigDecimalError(src) => { write!(f, "{}", src) }
            }
        }
    }


    // It can be generated by macro
    impl core::fmt::Display for ParseAmountError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{}", self.kind)
        }
    }

    impl core::fmt::Debug for ParseAmountError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            if self.backtrace.is_captured() {
                let src_contains_captured_backtrace: bool = crate::util::backtrace::BacktraceCopyProvider::contains_self_or_child_captured_backtrace(&self.source);
                if src_contains_captured_backtrace {
                    write!(f, "ParseAmountError {{ kind: {:?}, source: {:?} }}", self.kind, self.source)
                } else {
                    write!(f, "ParseAmountError {{ kind: {:?}, source: {:?}, backtrace: {} }}", self.kind, self.source, self.backtrace)
                }
            } else {
                write!(f, "ParseAmountError {{ kind: {:?}, source: {:?} }}", self.kind, self.source)
            }
        }
    }

    impl std::error::Error for ErrorSource {
        fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
            match self {
                ErrorSource::NoSource => { None }
                ErrorSource::CurrencyFormatError(ref? src) =>  { Some(src) }
                ErrorSource::ParseBigDecimalError(ref? src) => { Some(src) }
            }
        }

        #[allow(deprecated)]
        fn description(&self) -> &str {
            match self {
                ErrorSource::NoSource => { "" }
                ErrorSource::CurrencyFormatError(ref src) =>  { src.description() }
                ErrorSource::ParseBigDecimalError(ref src) => { src.description() }
            }
        }

        // fn provide<'a>(&'a self, request: &mut Request<'a>) { ... }
    }

    // impl BacktraceCopyProvider for ParseBigDecimalError {
    //     fn provide_backtrace(&self) -> BacktraceInfo { BacktraceInfo::empty() }
    // }
    */
}


#[cfg(test)]
mod tests {
    use crate::util::{TestOptionUnwrap, TestResultUnwrap};
    use crate::util::string::substring_count;
    use super::*;

    #[test]
    #[should_panic(expected = "ParseAmountError { kind: IncorrectAmount, source: ParseBigInt(ParseBigIntError { kind: InvalidDigit })")]
    fn test_parse_amount_01_01() {
        parse_amount_01(" \t \n 12_John_2.350 BRL ").test_unwrap();
    }

    #[test]
    #[should_panic(expected = "ParseAmountError { kind: IncorrectAmount, source: ParseBigInt(ParseBigIntError { kind: InvalidDigit })")]
    fn test_parse_amount_02_01() {
        parse_amount_02(" \t \n 12_John_2.350 BRL ").test_unwrap();
    }

    #[test]
    #[should_panic(expected = "ParseAmountError { kind: IncorrectAmount, source: ParseBigInt(ParseBigIntError { kind: InvalidDigit })")]
    fn test_parse_amount_03_01() {
        parse_amount_03(" \t \n 12_John_2.350 BRL ").test_unwrap();
    }

    #[test]
    fn test_from_anyhow_error() {
        let anyhow_err: anyhow::Error = anyhow::Error::msg("Error 123");
        let my_err = parse_amount_old::ParseAmountError::with_source(
            parse_amount_old::ErrorKind::IncorrectAmount,
            parse_amount_old::ErrorSource::SomeAnyHowError(anyhow_err));
        println!("my_err: {}", my_err);
        println!("-------------------------------------------");
        println!("my_err: {:?}", my_err);

        println!("\n\n-------------------------------------------");
        let io_err_res = std::fs::read_to_string("unknown_file.txt");
        let anyhow_err: anyhow::Error = From::from(io_err_res.err().test_unwrap());
        let my_err = parse_amount_old::ParseAmountError::with_source(
            parse_amount_old::ErrorKind::IncorrectAmount,
            parse_amount_old::ErrorSource::SomeAnyHowError(anyhow_err));
        println!("my_err: {}", my_err);
        println!("-------------------------------------------");
        println!("my_err: {:?}", my_err);

        let mut str_buf = String::new();
        use std::fmt::Write;
        write!(str_buf, "{:?}", my_err).test_unwrap();

        // ascii_substring_count(str_buf.as_str(), b"backtrace:");
        let count = substring_count(str_buf.as_str(), " backtrace:");
        assert_eq!(count, 1);
    }

    #[test]
    fn test_from_std_error() {
        println!("\n\n-------------------------------------------");

        let io_err_res = std::fs::read_to_string("unknown_file.txt");
        let io_err = io_err_res.err().test_unwrap();

        println!("io_err: {}", io_err);
        println!("-------------------------------------------");
        println!("io_err: {:?}", io_err);


        // -----------------------------------------------------------------------------------------
        let mut str_buf = String::new();
        use std::fmt::Write;
        write!(str_buf, "{:?}", io_err).test_unwrap();

        let count = substring_count(str_buf.as_str(), " backtrace:");
        // Now rust std errors do not print backtrace.
        // When it happens we need to make sure that we use it instead of capturing backtrace manually.
        assert_eq!(count, 0);


        let str_err_box: Box<dyn std::error::Error> = Box::new(io_err);
        let my_err = parse_amount_old::ParseAmountError::with_source(
            parse_amount_old::ErrorKind::IncorrectAmount,
            parse_amount_old::ErrorSource::StdError(str_err_box));
        println!("my_err: {}", my_err);
        println!("-------------------------------------------");
        println!("my_err: {:?}", my_err);

        let mut str_buf = String::new();
        write!(str_buf, "{:?}", my_err).test_unwrap();

        // ascii_substring_count(str_buf.as_str(), b"backtrace:");
        let count = substring_count(str_buf.as_str(), " backtrace:");
        assert_eq!(count, 1);
    }
}
