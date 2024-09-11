use core::str::FromStr;
use bigdecimal::BigDecimal;
use mvv_common::entity::amount::Amount;
use mvv_common::entity::currency::Currency;
use mvv_common::backtrace::backtrace;
use mvv_common::test::TestResultUnwrap;
//--------------------------------------------------------------------------------------------------


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
            source: ParseBigDecimalError(er), backtrace: backtrace() }) ?;

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
    use mvv_common::backtrace::BacktraceCell;
    use mvv_common::entity::currency::parse::CurrencyFormatError;

    #[derive(Debug, derive_more::Display)]
    #[display("Struct123")]
    pub struct Struct123;

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
    #[derive(mvv_error_macro::StructError)]
    #[do_not_generate_debug]
    // #[struct_error_internal_type_path_mode(InternalCratePath)]
    #[struct_error_internal_type_path_mode(ExternalCratePath)]
    pub struct ParseAmountError {
        pub kind: ErrorKind,
        #[source]
        // #[from]
        pub source: ErrorSource,
        pub backtrace: BacktraceCell,
    }

    //noinspection DuplicatedCode
    // It can be generated by macro
    #[derive(mvv_error_macro::StructErrorSource)]
    // Full type or short type can be used: ParseAmountError/crate::entity::amount::parse::ParseAmountError
    #[struct_error_type(ParseAmountError)]
    // #[struct_error_internal_type_path_mode(InternalCratePath)]
    #[struct_error_internal_type_path_mode(ExternalCratePath)]
    // #[do_not_generate_std_error]
    #[allow(dead_code)]
    pub enum ErrorSource {
        //#[error("No source")]
        NoSource,
        //#[error("Currency format error")]
        // #[error_macro::StaticStructErrorType(ParseAmountError)]
        #[from_error_kind(IncorrectCurrency)]
        CurrencyFormatError(CurrencyFormatError),
        // for testing
        // CurrencyFormatError22(crate::entity::currency::parse_currency::CurrencyFormatError),
        //#[error("Decimal format error")]
        #[from_error_kind(IncorrectAmount)]
        #[no_source_backtrace]
        ParseBigDecimalError(ParseBigDecimalError),

        // just for test
        SomeInt64(i64),

        SomeWithoutArg,

        // With duplicated types
        // #[error("Some1FromString")]
        // #[no_source_backtrace]
        // #[from_error_kind(IncorrectAmount)] // temp. to test proper 'duplicates' error
        Some1FromString(String),
        // #[error("Some2FromString")]
        // #[no_source_backtrace]
        // #[from_error_kind(IncorrectAmount)] // temp. to test proper 'duplicates' error
        Some2FromString(String),
        // #[error("Some1FromInt")]
        // #[no_source_backtrace]
        Some1FromInt(i32),
        // #[error("Some2FromInt")]
        #[no_source_backtrace]
        Some2FromInt(i32),
        #[no_source_backtrace]
        #[no_std_error]
        Some3FromSomeStruct(Struct123),

        //#[no_source_backtrace]
        // #[error("SomeAnyHowError")]
        SomeAnyHowError(anyhow::Error),

        #[no_source_backtrace] // TODO: impl and remove this 'no_source_backtrace'
        // #[error("SomeStdError")]
        StdError(Box<dyn std::error::Error>),
    }

    impl core::fmt::Debug for ParseAmountError {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            mvv_common::error::__private::error_debug_fmt_impl(
                f, self, "ParseAmountError", |er|&er.kind, |er|&er.source, |er|&er.backtrace)
        }
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
        #[track_caller]
        pub fn new(kind: ErrorKind) -> ParseAmountError {
            ParseAmountError { kind, source: ErrorSource::NoSource, backtrace: BacktraceInfo::new() }
        }
        // It can be generated by macro
        #[track_caller]
        pub fn with_backtrace(kind: ErrorKind, backtrace_policy: NewBacktracePolicy) -> ParseAmountError {
            ParseAmountError { kind, source: ErrorSource::NoSource, backtrace: BacktraceInfo::new_by_policy(backtrace_policy) }
        }
        // It can be generated by macro
        #[track_caller]
        pub fn with_source(kind: ErrorKind, source: ErrorSource) -> ParseAmountError {
            ParseAmountError { kind, backtrace: BacktraceInfo::inherit_from(&source), source }
        }
        // It can be generated by macro
        #[track_caller]
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
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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
                let src_contains_captured_backtrace: bool = mvv_common::backtrace::BacktraceCopyProvider::contains_self_or_child_captured_backtrace(&self.source);
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
    use assertables::{ assert_contains, assert_contains_as_result };
    use assertables::{ assert_starts_with, assert_starts_with_as_result };
    use mvv_common::test::{ TestOptionUnwrap, TestResultUnwrap };
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
        use core::fmt::Write;
        write!(str_buf, "{:?}", my_err).test_unwrap();

        assert_stack_trace_is_only_one(&str_buf);
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
        use core::fmt::Write;
        write!(str_buf, "{:?}", io_err).test_unwrap();

        assert_stack_trace_is_no_one(&str_buf);


        let str_err_box: Box<dyn std::error::Error> = Box::new(io_err);
        let my_err = parse_amount_old::ParseAmountError::with_source(
            parse_amount_old::ErrorKind::IncorrectAmount,
            parse_amount_old::ErrorSource::StdError(str_err_box));
        println!("my_err: {}", my_err);
        println!("-------------------------------------------");
        println!("my_err: {:?}", my_err);

        let mut str_buf = String::new();
        write!(str_buf, "{:?}", my_err).test_unwrap();

        assert_stack_trace_is_only_one(&str_buf);
    }


    #[test]
    fn test_print_error_source_with_primitive_type() {
        use parse_amount_old::*;
        use core::fmt::Write;

        let err = ParseAmountError::with_source(ErrorKind::NoCurrency, ErrorSource::Some1FromInt(852));

        let mut str_buf = String::new();
        write!(str_buf, "{}", err).test_unwrap();
        assert_eq!(str_buf, "ParseAmountError { No currency in amount }");
        // assert_eq!(str_buf, "ParseAmountError { No currency in amount, source: Some1FromInt( 852 ) }");
        // assert_contains!(str_buf, "Some1FromInt( 852 )");

        let mut str_buf = String::new();
        write!(str_buf, "{:?}", err).test_unwrap();
        assert_contains!(str_buf, "Some1FromInt(852)");
        assert_starts_with!(str_buf, "ParseAmountError { kind: NoCurrency, source: Some1FromInt(852)");

        let mut str_buf = String::new();
        write!(str_buf, "{}", err.source).test_unwrap();
        assert_contains!(str_buf, "Some1FromInt(852)");
        assert_eq!(str_buf, "Some1FromInt(852)");

        let mut str_buf = String::new();
        write!(str_buf, "{:?}", err.source).test_unwrap();
        assert_contains!(str_buf, "Some1FromInt(852)");
        assert_eq!(str_buf, "Some1FromInt(852)");
    }
}



////////////////////////////////////////////////////////////////////////////////////////////////////
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
#[allow(dead_code)]
fn assert_debug_stack_trace_is_no_one<Err: core::fmt::Debug>(err: &Err) {
    use core::fmt::Write;
    let mut str_buf = String::new();
    write!(str_buf, "{:?}", err).test_unwrap();
    assert_stack_trace_is_no_one(str_buf.as_str());
}
#[track_caller]
fn assert_stack_trace_is_only_one(str: &str) {
    let stack_trace_count = stack_trace_count(str);
    assert_eq!(stack_trace_count, 1,
               "Expected only 1 backtrace (but found {stack_trace_count}) [{str}]");
}
#[track_caller]
fn assert_stack_trace_is_no_one(str: &str) {
    let stack_trace_count = stack_trace_count(str);
    assert_eq!(stack_trace_count, 0, "There should not be any backtrace in [{str}]");
}

fn find_backtrace_index(str: &str) -> Option<usize> {
    str
        .find(" backtrace:")
        .or_else(|| str.find("backtrace: "))
        .or_else(|| str.find("backtrace:\t"))
        .or_else(|| str.find("backtrace:\n"))
        .or_else(|| str.find(" stacktrace:"))
        .or_else(|| str.find("stacktrace: "))
        .or_else(|| str.find("stacktrace:\t"))
        .or_else(|| str.find("stacktrace:\n"))
        .or_else(|| str.find(" stack trace:"))
        .or_else(|| str.find("stack trace: "))
        .or_else(|| str.find("stack trace:\t"))
        .or_else(|| str.find("stack trace:\n"))
}

fn stack_trace_count(str: &str) -> usize {
    let first_index: Option<usize> = find_backtrace_index(str);
    if first_index.is_none() {
        return 0;
    }

    let second_index: Option<usize> = first_index.and_then(|first_index| {
        let str: &str = &str[first_index + 3..];
        find_backtrace_index(str)
            .map(|i| i + first_index + 3)
    });

    if second_index.is_some() { 2 }
    else { 1 }
}
