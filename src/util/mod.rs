
pub mod result;
mod unchecked;
mod error;
mod json;

pub mod backtrace;
pub mod test_unwrap;
pub mod string;

pub use crate::util::result::{ as_printable, as_printable_ptr };
pub use crate::util::unchecked::{ UncheckedOptionUnwrap, UncheckedResultUnwrap };
pub use crate::util::test_unwrap::{ TestOptionUnwrap, TestResultUnwrap };

pub use crate::util::backtrace::{ BacktraceInfo, disable_backtrace, enable_backtrace, print_current_stack_trace };

pub use crate::util::error::{ ToAnyHowError, ToAnyHowErrorFn };
