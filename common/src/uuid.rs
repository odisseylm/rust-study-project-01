use crate::backtrace::{BacktraceCell, BacktraceSource};
use crate::{generate_backtrace_source_delegate, generate_display_delegate};
//--------------------------------------------------------------------------------------------------


#[derive(Debug, thiserror::Error)]
pub struct UuidFormatError {
    #[source]
    error: uuid::Error,
    backtrace: BacktraceCell,
}

generate_display_delegate! { UuidFormatError, error }
generate_backtrace_source_delegate! { UuidFormatError }

impl From<uuid::Error> for UuidFormatError {
    fn from(error: uuid::Error) -> Self {
        Self {
            error, backtrace: BacktraceCell::new()
        }
    }
}
