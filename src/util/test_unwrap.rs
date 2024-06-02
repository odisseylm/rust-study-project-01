use std::fmt::Debug;


// Actually this code is designed for unit test only,
// but in that case due to strange rust project tests build approach
// it causes showing 'unused code'.
// For that reason I've decided NOW to put it in prod code
// (probably later I'll move them back to 'tests' source directory and suppress
// and will add #[allow(dead_code)])


/// This trait and its impl was added to minimize uncontrolled usage of panic-risky unwrap.
/// Please
///  * use test_unwrap() in tests.
///  * use unchecked_unwrap() in 'xxx_unchecked' methods.
///
/// Try not use pure unwrap() at all production code (to avoid unpredictable panic).
///
pub trait TestResultUnwrap <Ok, Err: Debug> {
    fn test_unwrap(self) -> Ok;
}
pub trait TestOptionUnwrap <Ok> {
    fn test_unwrap(self) -> Ok;
}

impl<Ok,Err: Debug> TestResultUnwrap<Ok,Err> for Result<Ok,Err> {
    #[inline]
    #[track_caller]
    fn test_unwrap(self) -> Ok {
        self.unwrap() // allowed
    }
}

impl<Ok> TestOptionUnwrap<Ok> for Option<Ok> {
    #[inline]
    #[track_caller]
    fn test_unwrap(self) -> Ok {
        self.unwrap() // allowed
    }
}
