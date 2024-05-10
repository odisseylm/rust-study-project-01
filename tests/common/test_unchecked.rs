use std::fmt::Debug;

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
    fn test_unwrap(self) -> Ok {
        self.unwrap() // allowed
    }
}

impl<Ok> TestOptionUnwrap<Ok> for Option<Ok> {
    #[inline]
    fn test_unwrap(self) -> Ok {
        self.unwrap() // allowed
    }
}




#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unwrap_for_result_ok() {
        let result: Result<i32, &str> = Ok(123);
        assert_eq!(result.test_unwrap(), 123);
    }

    #[test]
    #[should_panic(expected = "Oops! Error 456.")]
    fn test_unwrap_for_result_error() {
        let result: Result<i32, &str> = Err("Oops! Error 456.");
        result.test_unwrap();
    }

    #[test]
    fn test_unwrap_for_option_ok() {
        assert_eq!(Some(123).test_unwrap(), 123);
    }

    #[test]
    #[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
    fn test_unwrap_for_option_none() {
        let option: Option<i32> = None;
        option.test_unwrap();
    }
}
