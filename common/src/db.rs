use core::fmt::{self, Debug, Display};
use std::cmp::min;
use crate::backtrace::{backtrace, BacktraceCell};
use crate::string::StaticRefOrString;


#[cfg(any(feature = "sqlx_07", feature = "sqlx_08"))]
mod pg;

#[cfg(feature = "sqlx_07")]
pub mod pg07;
#[cfg(feature = "sqlx_07")]
pub mod pg07_macro;

#[cfg(feature = "sqlx_08")]
pub mod pg08;
#[cfg(feature = "sqlx_08")]
pub mod pg08_macro;

pub mod diesel2;


//--------------------------------------------------------------------------------------------------
#[derive(thiserror::Error)]
#[derive(educe::Educe)] #[educe(Debug)]
pub enum DbMappingError<T: Display = EmptyDisplayStub> {
    #[error("UnexpectDbValue")]
    UnexpectDbValue {
        #[educe(Debug(method(print_only_part)))]
        value: T,
        table: StaticRefOrString,
        column: StaticRefOrString,
        backtrace: BacktraceCell,
    },
    #[error("IncorrectUt8DbValue")]
    IncorrectUtf8DbValue {
        #[educe(Debug(method(print_only_part)))]
        value: T,
        table: StaticRefOrString,
        column: StaticRefOrString,
        backtrace: BacktraceCell,
    },
}

#[derive(Debug)]
pub struct EmptyDisplayStub;
impl Display for EmptyDisplayStub {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <str as Display>::fmt("No data", f)
    }
}


impl<T: Display> DbMappingError<T> {
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

impl DbMappingError<String> {
    pub fn incorrect_utf_8_db_value(bytes: &[u8], table: &'static str, column: &'static str)
    -> DbMappingError<String> {
        const MAX_REPORT_BYTES_LEN: usize = 10;
        let report_bytes_len = min(bytes.len(), MAX_REPORT_BYTES_LEN);
        let as_lossy_str_prt = String::from_utf8_lossy(&bytes[0..report_bytes_len])
            .into_owned();

        DbMappingError::IncorrectUtf8DbValue {
            value: as_lossy_str_prt,
            table: table.into(),
            column: column.into(),
            backtrace: backtrace(),
        }
    }
}


fn print_only_part<T: Display>(value: &T, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{value:.10}...")
}



#[cfg(test)]
mod tests {
    use crate::test::TestDebugStringOps;
    use super::DbMappingError;

    #[test]
    fn print_only_part_of_data() {
        let err = DbMappingError::unexpect_db_value(
            "1234567890123456789LongLongLongLongLongLongLongLong",
            "TABLE1",
            "COLUMN1",
        );

        let debug_str = err.to_test_debug_string();
        // pretty_assertions::assert_eq!(debug_str, "dfdf");
        assert_text::assert_text_starts_with!(
            debug_str,
            r#"UnexpectDbValue { value: 1234567890..., table: Ref("TABLE1"), column: Ref("COLUMN1"), backtrace:"#);
    }
}