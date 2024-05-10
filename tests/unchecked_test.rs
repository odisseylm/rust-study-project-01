
use project01::util::{ UncheckedOptionUnwrap, UncheckedResultUnwrap };



#[test]
fn test_unchecked_unwrap_for_result_ok() {
    let result: Result<i32, &str> = Ok(123);
    assert_eq!(result.unchecked_unwrap(), 123);
}

#[test]
#[should_panic(expected = "Oops! Error 456.")]
fn test_unchecked_unwrap_for_result_error() {
    let result: Result<i32, &str> = Err("Oops! Error 456.");
    result.unchecked_unwrap();
}

#[test]
fn test_unchecked_unwrap_for_option_ok() {
    assert_eq!(Some(123).unchecked_unwrap(), 123);
}

#[test]
#[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
fn test_unchecked_unwrap_for_option_none() {
    let option: Option<i32> = None;
    option.unchecked_unwrap();
}
