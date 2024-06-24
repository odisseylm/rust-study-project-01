
use core::fmt::Debug;


/// This trait and its impl was added to minimize uncontrolled usage of panic-risky unwrap.
/// Please
///  * use unchecked_unwrap() in 'xxx_unchecked' methods.
///  * use test_unwrap() in tests.
///
/// Try not use pure unwrap() at all production code (to avoid unpredictable panic).
///
pub trait UncheckedResultUnwrap <Ok, Err: Debug> {
    fn unchecked_unwrap(self) -> Ok;
}
pub trait UncheckedOptionUnwrap <Ok> {
    fn unchecked_unwrap(self) -> Ok;
}

impl<Ok,Err: Debug> UncheckedResultUnwrap<Ok,Err> for Result<Ok,Err> {
    #[inline]
    fn unchecked_unwrap(self) -> Ok {
        self.unwrap() // allowed
    }
}

impl<Ok> UncheckedOptionUnwrap<Ok> for Option<Ok> {
    #[inline]
    fn unchecked_unwrap(self) -> Ok {
        self.unwrap() // allowed
    }
}
