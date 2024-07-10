use core::str::FromStr;
use serde::Deserialize;
use serde_with::serde_derive::Serialize;

#[derive(Debug, Clone, PartialEq, derive_more::Display)]
#[derive(Serialize, Deserialize)]
#[display(fmt = "{}", "0")]
// #[nutype(
//     validate(not_empty, len_char_max = 20),
//     derive(Debug, PartialEq),
// )]
pub struct Id(String);

impl Id {
    pub fn move_out(self) -> String { self.0 }
}


#[inherent::inherent]
impl FromStr for Id {
    type Err = parse::IdFormatError;

    pub fn from_str(str: &str) -> Result<Self, parse::IdFormatError> {
        // TODO: impl validation
        Ok(Id(str.to_string()))
    }
}



pub mod parse {
    use crate::util::backtrace::BacktraceInfo;

    #[derive(Debug, thiserror::Error)]
    #[derive(Copy, Clone)]
    pub enum ErrorKind {
        #[error("Incorrect ID format")]
        IncorrectIdFormat
    }

    #[derive(thiserror::Error)]
    #[derive(static_error_macro::MyStaticStructError)]
    pub struct IdFormatError {
        pub kind: ErrorKind,
        pub backtrace: BacktraceInfo,
    }
}
