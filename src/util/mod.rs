
pub mod result;
mod unchecked;
pub mod error;
mod json;

pub mod backtrace;
pub mod test_unwrap;
pub mod string;
pub mod obj_ext;
pub mod fmt;
pub mod serde_json;
mod other;

pub use crate::util::result::{ as_printable, as_printable_ptr };
pub use crate::util::unchecked::{ UncheckedOptionUnwrap, UncheckedResultUnwrap };
pub use crate::util::test_unwrap::{ TestOptionUnwrap, TestResultUnwrap };

pub use crate::util::backtrace::{ BacktraceInfo, disable_backtrace, enable_backtrace, print_current_stack_trace };

pub use crate::util::error::{ ToAnyHowError, ToAnyHowErrorFn };

pub mod env {
    pub use super::other::env_var;
    pub use super::other::env_var_or_else;
    pub use crate::util::other::EnvVarOps;
}
