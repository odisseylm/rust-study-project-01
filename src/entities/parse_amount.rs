use std::str::FromStr;
use bigdecimal::BigDecimal;
use crate::entities::amount::Amount;
use crate::entities::currency::Currency;


// Just example
#[path = "./dir1/dir2/some_relative_path_01.rs"]
mod relative_welcome_home;

#[allow(dead_code)]
fn usage_nf_from_relative_path() {
    relative_welcome_home::fn_from_rs_path_01()
}


// For internal usage, Use Amount::from_str() in production code.
//
pub fn parse_amount(s: &str) -> Result<Amount, ParseAmountError> {
    // use crate::entities::amount::parse_amount::{ParseAmountError, ErrorKind };

    let s = s.trim();

    let last_space_bytes_offset = s.rfind(|ch: char|{ ch.is_ascii_whitespace() })
        .ok_or_else(|| ParseAmountError::new(ErrorKind::NoCurrency)) ?;

    let (str_amount, str_cur) = s.split_at(last_space_bytes_offset);

    let currency = Currency::from_str(str_cur.trim_start()) ?;
    let amount = BigDecimal::from_str(str_amount.trim_end()) ?;

    Ok(Amount::new(amount, currency))
}


// -------------------------------------------------------------------------------------------------
//                                        Error
// -------------------------------------------------------------------------------------------------


// rust does not support nested structs/types/so on.
// As workaround, I decided to use sub-namespace.
//
//noinspection DuplicatedCode  // duplicated since copy of this code (in amount_parse_old.rs) is also used for testing MyStaticStructError
//pub mod parse_amount {

use bigdecimal::ParseBigDecimalError;
use crate::util::BacktraceInfo;
use crate::entities::currency::parse_currency::CurrencyFormatError;


// Duplicated since copy of this code (in amount_parse_old.rs) is also used for testing MyStaticStructError
// noinspection DuplicatedCode
//
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


// Duplicated since copy of this code (in amount_parse_old.rs) is also used for testing MyStaticStructError
// noinspection DuplicatedCode
//
#[derive(thiserror::Error)]
#[derive(static_error_macro::MyStaticStructError)]
// #[do_not_generate_debug]
pub struct ParseAmountError {
    pub kind: ErrorKind,
    #[source]
    // #[from]
    pub source: ErrorSource,
    pub backtrace: BacktraceInfo,
}


// Duplicated code since copy of this code (in amount_parse_old.rs) is also used for testing MyStaticStructError
// noinspection DuplicatedCode
//
// #[derive(thiserror::Error)]
#[derive(static_error_macro::MyStaticStructErrorSource)]
#[struct_error_type(ParseAmountError)]
// #[derive(Debug)]
pub enum ErrorSource {
    // #[error("No source")]
    NoSource,

    // #[error("Currency format error")]
    #[from_error_kind(IncorrectCurrency)]
    CurrencyFormatError(CurrencyFormatError),

    // #[error("Decimal format error")]
    #[from_error_kind(IncorrectAmount)]
    #[no_source_backtrace]
    ParseBigDecimalError(ParseBigDecimalError),
}


impl std::error::Error for ErrorSource {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ErrorSource::NoSource => { None }
            ErrorSource::CurrencyFormatError(src) => { Some(src) }
            ErrorSource::ParseBigDecimalError(src) => { Some(src) }
        }
    }

    #[allow(deprecated)]
    fn description(&self) -> &str {
        match self {
            ErrorSource::NoSource => { "" }
            ErrorSource::CurrencyFormatError(src) => { src.description() }
            ErrorSource::ParseBigDecimalError(src) => { src.description() }
        }
    }

    // fn provide<'a>(&'a self, request: &mut Request<'a>) { ... }
}


// }
