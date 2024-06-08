use std::str::FromStr;
use serde::Deserialize;
use serde_with::serde_derive::Serialize;

#[derive(Debug, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(PartialEq)]
// #[nutype(
//     validate(not_empty, len_char_max = 20),
//     derive(Debug, PartialEq),
// )]
pub struct Id(String);

impl Id {
}


#[inherent::inherent]
impl FromStr for Id {
    type Err = parse::IdFormatError;

    pub fn from_str(str: &str) -> Result<Self, parse::IdFormatError> {
        // TODO: impl validation
        Ok(Id(str.to_string()))
    }
}

// TODO: use nu-type crate
impl core::fmt::Display for Id {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
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
