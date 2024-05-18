use std::str::{ FromStr };
use std::fmt::{ Display, Formatter };
use bigdecimal::{ BigDecimal, BigDecimalRef, ParseBigDecimalError };
use crate::entities::{ Currency, CurrencyFormatError };
use crate::util::{BacktraceInfo, UncheckedResultUnwrap};
use crate::util::backtrace::BacktraceCopyProvider;


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
        self.currency.clone()
    }

    pub fn currency_ref(&self) -> &Currency {
        &self.currency
    }

    pub fn value(&self) -> &BigDecimal {
        &self.value
    }

    pub fn value_ref(&self) -> BigDecimalRef<'_> {
        self.value.to_ref()
    }

    pub fn with_value(&self, amount: BigDecimal) -> Amount {
        Amount { value: amount, currency: self.currency }
    }

    // I do not see sense to have function to 'change' currency (there are no such user/bank cases).
    // pub fn with_currency(&self, currency: Currency) -> Amount {
    //     Amount { value: self.value.clone(), currency }
    // }
}

#[inline]
pub fn amount(amount: BigDecimal, currency: Currency) -> Amount { Amount::new(amount, currency) }


fn parse_amount(s: &str) -> Result<Amount, ParseAmountError> {
    use ErrorSource::*;

    let s = s.trim();

    let last_space_bytes_offset = s.rfind(|ch: char|{ ch.is_ascii_whitespace() })
        .ok_or( ParseAmountError::new(ParseAmountErrorKind::NoCurrencyError)) ?;

    let (str_amount, str_cur) = s.split_at(last_space_bytes_offset);

    let currency = Currency::from_str(str_cur.trim_start())
        .map_err(|er| {
            ParseAmountError::with_source(
                ParseAmountErrorKind::ParseCurrencyError,
                CurrencyFormatError(er),
            )
        }) ?;

    let amount = BigDecimal::from_str(str_amount.trim_end())
        .map_err(|er| ParseAmountError {
            kind: ParseAmountErrorKind::ParseAmountError,
            source: ParseBigDecimalError(er),
            backtrace: BacktraceInfo::new(),
        }) ?;

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



/*
#[derive(Debug, PartialEq, Clone)]
pub enum ParseAmountError {
    NoCurrencyError,
    ParseCurrencyError(CurrencyFormatError),
    ParseAmountError(ParseBigDecimalError),
}
*/

#[derive(Debug, thiserror::Error)]
pub enum ParseAmountErrorKind { // TODO: try to do as nested one
    #[error("No currency in amount error")]
    NoCurrencyError,
    #[error("Currency format error")]
    ParseCurrencyError,
    #[error("Parse amount value error")]
    ParseAmountError,
}

#[derive(Debug, thiserror::Error)]
pub struct ParseAmountError {
    pub kind: ParseAmountErrorKind,
    #[source]
    pub source: ErrorSource,
    pub backtrace: BacktraceInfo,

    // enum ErrorKind {
    //     ONE,
    // }
}


impl ParseAmountError {
    pub fn new(kind: ParseAmountErrorKind) -> ParseAmountError {
        ParseAmountError { kind, source: ErrorSource::NoSource, backtrace: BacktraceInfo::new() }
    }
    pub fn with_source(kind: ParseAmountErrorKind, source: ErrorSource) -> ParseAmountError {
        ParseAmountError { kind, backtrace: BacktraceInfo::inherit_from(&source), source }
    }
}

#[derive(thiserror::Error)]
pub enum ErrorSource {
    #[error("No source")]
    NoSource,
    #[error("Currency format error")]
    CurrencyFormatError(CurrencyFormatError),
    #[error("Decimal format error")]
    ParseBigDecimalError(ParseBigDecimalError),
}

impl BacktraceCopyProvider for ErrorSource {
    fn provide_backtrace(&self) -> BacktraceInfo {
        match self {
            ErrorSource::NoSource => { BacktraceInfo::empty() }
            ErrorSource::ParseBigDecimalError(_)  => { BacktraceInfo::empty() }
            ErrorSource::CurrencyFormatError(src) => { src.provide_backtrace() }
        }
    }
}

impl core::fmt::Debug for ErrorSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use ErrorSource::*;
        match self {
            NoSource                      => { write!(f, "No source") }
            CurrencyFormatError(ref src)  => { write!(f, "{:?}", src) }
            ParseBigDecimalError(ref src) => { write!(f, "{:?}", src) }
        }
    }
}


// From the RFC
// struct RectangleTidy {
//     dimensions: {
//         width: u64,
//         height: u64,
//     },
//     color: {
//         red: u8,
//         green: u8,
//         blue: u8,
//     },
// }


impl Display for ParseAmountError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl FromStr for Amount {
    type Err = ParseAmountError;

    #[inline]
    fn from_str(s: &str) -> Result<Amount, Self::Err> { parse_amount(s) }
}
