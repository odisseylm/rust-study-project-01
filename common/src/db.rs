use core::fmt::{Debug};
use crate::backtrace::{backtrace, BacktraceCell};
use crate::string::StaticRefOrString;

pub mod pg;
pub mod diesel2;

//--------------------------------------------------------------------------------------------------
#[derive(Debug, thiserror::Error)]
pub enum DbMappingError<T: Debug = ()> {
    #[error("UnexpectDbValue")]
    UnexpectDbValue {
        value: T, // TODO: in Debug use only restricted number of chars
        column: StaticRefOrString,
        table: StaticRefOrString,
        backtrace: BacktraceCell,
    },
    #[error("IncorrectUt8DbValue")]
    IncorrectUt8DbValue {
        value: T, // TODO: in Debug use only restricted number of chars
        column: StaticRefOrString,
        table: StaticRefOrString,
        backtrace: BacktraceCell,
    },
}

impl<T: Debug> DbMappingError<T> {
    pub fn unexpect_db_value(value: T, table: &'static str, column: &'static str) -> DbMappingError<T> {
        DbMappingError::<T>::UnexpectDbValue {
            value,
            table: table.into(),
            column: column.into(),
            backtrace: backtrace(),
        }
    }
}
impl DbMappingError<fixedstr::str32> {
    // Using fixedstr::str8 instead of standard String it is just experiment/investigation,
    // like tiny optimization to avoid using heap if it can be skipped.
    //
    pub fn unexpect_db_tiny_str(value: &str, table: &'static str, column: &'static str)
        -> DbMappingError<fixedstr::str32> {

        let value_to_report = fixedstr::str32::make(value); // chars/bytes after capacity (32) are ignored.
        DbMappingError::<fixedstr::str32>::UnexpectDbValue {
            value: value_to_report,
            table: table.into(),
            column: column.into(),
            backtrace: backtrace(),
        }
    }
}
