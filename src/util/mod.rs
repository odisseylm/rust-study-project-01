
pub mod result;
mod unchecked;
mod error;
mod json;

pub mod backtrace;
pub mod test_unwrap;

pub use crate::util::result::{ as_printable, as_printable_ptr };
pub use crate::util::unchecked::{ UncheckedOptionUnwrap, UncheckedResultUnwrap };
pub use crate::util::test_unwrap::{ TestOptionUnwrap, TestResultUnwrap };

pub use crate::util::json::{ Entity1, error_fn_5, extract_json, extract_json_5, MyError333, MyError334 };
pub use crate::util::json::{ error_fn_105, error_fn_205, error_fn_305, error_fn_405, error_fn_505, error_fn_605, error_fn_705 };

pub use crate::util::backtrace::{ BacktraceInfo, disable_backtrace, enable_backtrace, print_current_stack_trace };

pub use crate::util::error::{ ToAnyHowError, ToAnyHowErrorFn };
