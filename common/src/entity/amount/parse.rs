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
//noinspection DuplicatedCode  // duplicated since copy of this code (in amount_parse_old.rs) is also used for testing StructError
//pub mod parse_amount {

use bigdecimal::ParseBigDecimalError;
use crate::backtrace::BacktraceCell;
use crate::entity::currency::parse::CurrencyFormatError;
use crate::entity::error::DataFormatError;

// Duplicated since copy of this code (in amount_parse_old.rs) is also used for testing StructError
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


// Duplicated since copy of this code (in amount_parse_old.rs) is also used for testing StructError
// noinspection DuplicatedCode
//
#[derive(thiserror::Error)]
#[derive(mvv_error_macro::StructError)]
// #[do_not_generate_debug]
pub struct AmountFormatError {
    pub kind: ErrorKind,
    #[source]
    pub source: ErrorSource,
    pub backtrace: BacktraceCell,
}

impl DataFormatError for AmountFormatError { }

// Duplicated code since copy of this code (in amount_parse_old.rs) is also used for testing StructError
// noinspection DuplicatedCode
//
// #[derive(thiserror::Error)]
#[derive(mvv_error_macro::StructErrorSource)]
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
