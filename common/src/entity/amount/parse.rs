use core::str::FromStr;
use bigdecimal::BigDecimal;
use crate::entity::{ amount::Amount, currency::Currency };
//--------------------------------------------------------------------------------------------------



// For internal usage, Use Amount::from_str() in production code.
//
pub fn parse_amount(s: &str) -> Result<Amount, AmountFormatError> {

    let s = s.trim();

    let last_space_bytes_offset = s.rfind(|ch: char|{ ch.is_ascii_whitespace() })
        .ok_or_else(|| AmountFormatError::new(ErrorKind::NoCurrency)) ?;

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
use crate::backtrace2::BacktraceCell;
use crate::entity::currency::parse::CurrencyFormatError;


// Duplicated since copy of this code (in amount_parse_old.rs) is also used for testing MyStaticStructError
// noinspection DuplicatedCode
//
#[derive(Debug, thiserror::Error, PartialEq)]
#[derive(Copy, Clone)]
pub enum ErrorKind {
    #[error("No currency in amount")]
    NoCurrency,
    #[error("Incorrect currency format")]
    IncorrectCurrency,
    #[error("No amount in amount")]
    NoAmount,
    #[error("Incorrect amount format")]
    IncorrectAmount,
}


// Duplicated since copy of this code (in amount_parse_old.rs) is also used for testing MyStaticStructError
// noinspection DuplicatedCode
//
#[derive(thiserror::Error)]
#[derive(mvv_static_error_macro::MyStaticStructError)]
// #[do_not_generate_debug]
pub struct AmountFormatError {
    pub kind: ErrorKind,
    #[source]
    // #[from]
    pub source: ErrorSource,
    pub backtrace: BacktraceCell,
}


// Duplicated code since copy of this code (in amount_parse_old.rs) is also used for testing MyStaticStructError
// noinspection DuplicatedCode
//
// #[derive(thiserror::Error)]
#[derive(mvv_static_error_macro::MyStaticStructErrorSource)]
#[struct_error_type(AmountFormatError)]
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


// }
