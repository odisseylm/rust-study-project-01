
pub mod result;
mod unchecked;

pub use crate::util::result::{ as_printable, as_printable_ptr };
pub use crate::util::unchecked::{ UncheckedOptionUnwrap, UncheckedResultUnwrap };
