use std::str::{ FromStr };
use std::fmt::{ Display, Formatter };
use bigdecimal::{ BigDecimal, BigDecimalRef, ParseBigDecimalError };
use crate::entities::currency::Currency;
use crate::util::UncheckedResultUnwrap;


#[derive(Debug)]
pub struct Amount {
    value: BigDecimal,
    currency: Currency,
}

impl Display for Amount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.value, self.currency)
    }
}


impl Amount {

    pub fn with_str_amount(amount: &str, currency: Currency) -> Result<Self, ParseBigDecimalError> {
        let bd: Result<BigDecimal, ParseBigDecimalError> = BigDecimal::from_str(amount);
        return bd.map(|am| Amount { value: am, currency } );
    }

    pub fn with_str_amount_unchecked(amount: &str, currency: Currency) -> Self {
        Amount::with_str_amount(amount, currency).unchecked_unwrap()
    }

    #[inline]
    pub fn new(amount: BigDecimal, currency: Currency) -> Amount {
        Amount { value: amount, currency }
    }

    // fn from_string(amount_with_currency: &str)

    pub fn currency(&self) -> Currency {
        self.currency
    }

    // I do not see sense to have it since copy of currency is cheap.
    // pub fn currency_ref(&self) -> &Currency { &self.currency }

    pub fn value_ref(&self) -> &BigDecimal {
        &self.value
    }

    pub fn value_bd_ref(&self) -> BigDecimalRef<'_> {
        self.value.to_ref()
    }

    pub fn with_value(&self, amount: BigDecimal) -> Amount {
        Amount { value: amount, currency: self.currency }
    }

    // I do not see sense to have function to 'change' currency (there are no such user/bank cases).
    // Having such method just can provoke making incorrect/senseless operations.
    // pub fn with_currency(&self, currency: Currency) -> Amount {
    //     Amount { value: self.value.clone(), currency }
    // }
}

// Just short alias (similar to kotlin style)
#[inline]
pub fn amount(amount: BigDecimal, currency: Currency) -> Amount { Amount::new(amount, currency) }


fn parse_amount(s: &str) -> Result<Amount, parse_amount::ParseAmountError> {
    // use parse_amount::ErrorSource::*;
    use parse_amount::{ ParseAmountError, ErrorKind };

    let s = s.trim();

    let last_space_bytes_offset = s.rfind(|ch: char|{ ch.is_ascii_whitespace() })
        .ok_or( ParseAmountError::new(ErrorKind::NoCurrency)) ?;

    let (str_amount, str_cur) = s.split_at(last_space_bytes_offset);

    let currency = Currency::from_str(str_cur.trim_start())
        // .map_err(|er| {
        //     // ParseAmountError::with_source(
        //     //     ErrorKind::IncorrectCurrency,
        //     //     CurrencyFormatError(er),
        //     // )
        //     // or
        //     ParseAmountError::with_from(ErrorKind::IncorrectCurrency, er)
        // })
        ?;

    let amount = BigDecimal::from_str(str_amount.trim_end())
        // .map_err(|er|
        //     // ParseAmountError {
        //     //     kind: ErrorKind::IncorrectAmount,
        //     //     source: ParseBigDecimalError(er),
        //     //     backtrace: BacktraceInfo::new(),
        //     // }
        //     // or
        //     ParseAmountError::with_from(ErrorKind::IncorrectAmount, er)
        // )
        ?;

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

impl FromStr for Amount {
    type Err = parse_amount::ParseAmountError;

    #[inline]
    fn from_str(s: &str) -> Result<Amount, Self::Err> { parse_amount(s) }
}


// -------------------------------------------------------------------------------------------------
//                                        Error
// -------------------------------------------------------------------------------------------------


// rust does not support nested structs/types/so on.
// As workaround, I decided to use sub-namespace.
//
pub mod parse_amount {
    use bigdecimal::ParseBigDecimalError;
    use crate::util::BacktraceInfo;
    use crate::entities::currency::parse_currency::CurrencyFormatError;

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

    #[derive(Debug, thiserror::Error)]
    #[derive(other01::MyStaticStructError)]
    pub struct ParseAmountError {
        pub kind: ErrorKind,
        #[source]
        // #[from]
        pub source: ErrorSource,
        pub backtrace: BacktraceInfo,
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
    */

    // It can be generated by macro
    #[derive(thiserror::Error)]
    #[derive(other01::MyStaticStructErrorSource)]
    // Full type or short type can be used: ParseAmountError/crate::entities::amount::parse_amount::ParseAmountError
    #[struct_error_type(ParseAmountError)]
    pub enum ErrorSource {
        #[error("No source")]
        NoSource,
        #[error("Currency format error")]
        // #[other01::StaticStructErrorType(ParseAmountError)]
        #[from_error_kind(IncorrectCurrency)]
        CurrencyFormatError(CurrencyFormatError),
        // for testing
        // CurrencyFormatError22(crate::entities::currency::parse_currency::CurrencyFormatError),
        #[error("Decimal format error")]
        #[from_error_kind(IncorrectAmount)]
        #[no_source_backtrace]
        ParseBigDecimalError(ParseBigDecimalError),
    }


    /*
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
