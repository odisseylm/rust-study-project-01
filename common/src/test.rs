use core::fmt::Debug;
use std::fmt::Display;
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


pub trait TestDisplayStringOps {
    #[track_caller]
    fn to_test_display_string(&self) -> String;
    #[track_caller]
    fn to_test_string(&self) -> String;
}

pub trait TestOptionDisplayStringOps {
    #[track_caller]
    fn to_test_display_string(&self) -> String;
    #[track_caller]
    fn to_test_string(&self) -> String;
}

pub trait TestDebugStringOps {
    #[track_caller]
    fn to_test_debug_string(&self) -> String;
}

impl<T> TestDisplayStringOps for T where T: Display {
    #[track_caller]
    fn to_test_display_string(&self) -> String {
        let mut str_buf = String::new();
        use core::fmt::Write;
        write!(str_buf, "{}", self).test_unwrap();
        str_buf
    }
    #[track_caller]
    fn to_test_string(&self) -> String {
        self.to_string()
    }
}

impl<T> TestOptionDisplayStringOps for Option<T> where T: Display {
    #[track_caller]
    fn to_test_display_string(&self) -> String {
        match self {
            None => "None".to_owned(),
            Some(ref v) => {
                let mut str_buf = String::new();
                use core::fmt::Write;
                write!(str_buf, "{}", v).test_unwrap();
                str_buf
            }
        }
    }
    #[track_caller]
    fn to_test_string(&self) -> String {
        match self {
            None => "None".to_owned(),
            Some(ref v) => v.to_string()
        }
    }
}

/*
impl<T> TestOptionDisplayStringOps for Option<&T> where T: Display {
    #[track_caller]
    fn to_test_display_string(&self) -> String {
        match self {
            None => "None".to_owned(),
            Some(ref v) => {
                let mut str_buf = String::new();
                use core::fmt::Write;
                write!(str_buf, "{}", v).test_unwrap();
                str_buf
            }
        }
    }
    #[track_caller]
    fn to_test_string(&self) -> String {
        self.to_string()
    }
}
*/

impl<T> TestDebugStringOps for T where T: Debug {
    #[track_caller]
    fn to_test_debug_string(&self) -> String {
        let mut str_buf = String::new();
        use core::fmt::Write;
        write!(str_buf, "{:?}", self).test_unwrap();
        str_buf
    }
}


#[extension_trait::extension_trait]
pub impl<T> TestOps for T where T: Clone {
    fn test_clone(&self) -> Self {
        self.clone()
    }
}

#[extension_trait::extension_trait]
pub impl<V,E> TestResultDebugErrOps for Result<V,E> where E: Debug {
    // #[inline] // warning: `#[inline]` is ignored on function prototypes
    #[track_caller]
    fn err_to_test_debug_string(self) -> String {
        self.err().test_unwrap().to_test_debug_string()
    }
}

#[extension_trait::extension_trait]
pub impl<V,E> TestResultDisplayErrOps for Result<V,E> where E: core::fmt::Display {
    // #[inline] // warning: `#[inline]` is ignored on function prototypes
    #[track_caller]
    fn err_to_test_display_string(self) -> String {
        // self.err().test_unwrap().to_test_display_string()

        let err = self.err().test_unwrap();
        let mut str_buf = String::new();
        use core::fmt::Write;
        write!(str_buf, "{}", err).test_unwrap();
        str_buf
    }
}
