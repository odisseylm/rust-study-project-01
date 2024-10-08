use core::fmt::{ Debug, Display };


// optional marker trait
pub trait DataFormatError: Display + Debug + std::error::Error { }

impl DataFormatError for mvv_common::uuid::UuidFormatError { }
impl DataFormatError for uuid::Error { }
impl DataFormatError for iban::ParseError { }