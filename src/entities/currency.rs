// #![macro_use]

use core::fmt;
use std::char;
use std::fmt::Formatter;
use std::str::FromStr;
use crate::util::backtrace::BacktraceCopyProvider;
use crate::util::BacktraceInfo;


// #[derive(Debug, PartialEq, Copy, Clone)]
#[derive(Debug, thiserror::Error)]
pub struct CurrencyFormatError {
    kind: CurrencyFormatErrorKind,
    backtrace: BacktraceInfo,
}

impl CurrencyFormatError {
    pub fn new(kind: CurrencyFormatErrorKind) -> CurrencyFormatError {
        CurrencyFormatError { kind, backtrace: BacktraceInfo::new() }
    }
    // pub fn with_backtrace(kind: CurrencyFormatErrorKind, backtrace_policy: NewBacktracePolicy) -> CurrencyFormatError {
    //     CurrencyFormatError { kind, backtrace: BacktraceInfo::new(backtrace_policy) }
    // }
    // pub fn without_backtrace(kind: CurrencyFormatErrorKind) -> CurrencyFormatError {
    //     CurrencyFormatError { kind, backtrace: BacktraceInfo::empty() }
    // }
    pub fn kind(&self) -> &CurrencyFormatErrorKind { &self.kind }
    pub fn backtrace(&self) -> &BacktraceInfo { &self.backtrace }
    // pub fn backtrace_copy(&self) -> BacktraceInfo { self.backtrace.clone() }
}


impl BacktraceCopyProvider for CurrencyFormatError {
    fn provide_backtrace(&self) -> BacktraceInfo { self.backtrace.clone() }
}


impl fmt::Display for CurrencyFormatError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "CurrencyFormatError  {}", self.kind)
    }
}


// #[derive(Debug, PartialEq, Copy, Clone)]
#[derive(Debug, thiserror::Error)]
#[derive(Copy, Clone)]
pub enum CurrencyFormatErrorKind {
    #[error("no currency (blank string)")]
    NoCurrency,
    #[error("Currency format error")]
    CurrencyFormatError,
}


#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Currency([u8;3]);


impl Currency {
    pub fn from_str(currency_code: & str) -> Result<Self, CurrencyFormatError> {
        if currency_code.is_empty() {
            return Err(CurrencyFormatError::new(CurrencyFormatErrorKind::NoCurrency))
        }

        let is_valid = if currency_code.len() != 3 { false }
                     else { is_valid_currency_ascii(currency_code.as_bytes()) };

        if !is_valid { Err(CurrencyFormatError::new(CurrencyFormatErrorKind::CurrencyFormatError)) }
        else {
            let as_bytes = currency_code.as_bytes();
            Ok(Self([as_bytes[0], as_bytes[1], as_bytes[2]]))
        }
    }

    pub fn new(currency_code: String) -> Result<Self, CurrencyFormatError> {
        Currency::from_str(currency_code.as_str())
    }

    pub fn code_as_ascii_bytes(&self) -> [u8;3] {
        self.0
    }

    pub fn code_as_string(&self) -> String {
        // a bit overcomplicated and needs risky unwrap...
        //String::from_utf8(Vec::from(self.0))

        let mut s: String = String::with_capacity(3);
        s.push(self.0[0] as char);
        s.push(self.0[1] as char);
        s.push(self.0[2] as char);
        return s;
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}{}", self.0[0] as char, self.0[1] as char, self.0[2] as char)
    }
}


impl FromStr for Currency {
    type Err = CurrencyFormatError;

    #[inline]
    fn from_str(s: &str) -> Result<Currency, Self::Err> {
        Currency::from_str(s)
    }
}



const fn const_panic_wrong_currency(_currency: & 'static str) -> ! {
    // It would be nice to print 'currency' in some way,
    // but seems it is impossible in const/inline functions, Use macro for it.
    panic!("Invalid currency (It should be 3 UPPERCASE english letters).")
}
const fn const_panic_wrong_currency_ascii(_currency: & 'static [u8]) -> ! {
    // It would be nice to print 'currency' in some way,
    // but seems it is impossible in const/inline functions, Use macro for it.
    panic!("Invalid currency (It should be 3 UPPERCASE english letters).")
}

#[inline]
const fn is_valid_cur_char(ch: char) -> bool {
    ch.is_ascii_alphabetic() && ch.is_ascii_uppercase()
}
#[inline]
const fn is_valid_cur_byte(ch: u8) -> bool { is_valid_cur_char(ch as char) }

// hm... like kotlin inline dependent functions, this validate_currency should be also published.
const fn is_valid_currency_ascii(cur: &[u8]) -> bool {
    if cur.len() != 3 { false }
    else { is_valid_cur_byte(cur[0]) && is_valid_cur_byte(cur[1]) && is_valid_cur_byte(cur[2]) }
}


/// Creates currency.
/// in case of wrong input a panic will be thrown.
///
/// # Examples
/// ```
/// use project01::entities::{ Currency, make_currency };
/// const PLN: Currency = make_currency("PLN");
/// assert_eq!(PLN.code_as_string(), "PLN");
/// assert_eq!(PLN.code_as_ascii_bytes(), *b"PLN");
/// ```
/// ```rust,should_panic
/// use project01::entities::{ Currency, make_currency };
///
/// // lowercase - error, in case of 'const' there will be compilation error.
/// // The best approach!!! (but not for tests)
/// // const PLN: Currency = make_currency("pln");
/// make_currency("pln"); // lowercase
/// ```
///
pub const fn make_currency(currency_code: & 'static str) -> Currency {
    let is_valid = if currency_code.len() != 3 { false }
                 else { is_valid_currency_ascii(currency_code.as_bytes()) };
    if !is_valid { const_panic_wrong_currency(currency_code) }
    let bytes = currency_code.as_bytes();
    return Currency([bytes[0], bytes[1], bytes[2]]);
}



/// Creates currency.
/// in case of wrong input a panic will be thrown.
///
/// # Examples
/// ```
/// use project01::entities::{ Currency, make_currency_b };
/// const PLN: Currency = make_currency_b(b"PLN");
/// assert_eq!(PLN.code_as_string(), "PLN");
/// assert_eq!(PLN.code_as_ascii_bytes(), *b"PLN");
/// ```
/// ```rust,should_panic
/// use project01::entities::{ Currency, make_currency_b };
/// // lowercase - error, in case of 'const' there will be compilation error.
/// // The best approach!!! (but not for tests)
/// // const PLN: Currency = make_currency_b(b"pln");
/// make_currency_b(b"pln"); // lowercase
/// ```
///
pub const fn make_currency_b(cur: & 'static [u8;3]) -> Currency {
    let is_valid = is_valid_currency_ascii(cur);
    if !is_valid { const_panic_wrong_currency_ascii(cur) }
    return Currency([cur[0], cur[1], cur[2]]);
}



/// Creates currency.
/// in case of wrong input a panic will be thrown.
///
/// # Examples
/// ```
/// use project01::entities::Currency;
/// use project01::make_currency; // macro
/// use project01::entities::currency::make_currency; // required inline function
///
/// const PLN: Currency = make_currency!("PLN");
/// assert_eq!(PLN.code_as_string(), "PLN");
/// assert_eq!(PLN.code_as_ascii_bytes(), *b"PLN");
/// ```
/// ```rust,should_panic
/// use project01::entities::Currency;
/// use project01::make_currency; // macro
/// use project01::entities::currency::make_currency; // required inline function
///
/// // lowercase - error, in case of 'const' there will be compilation error.
/// // The best approach!!! (but not for tests)
/// // const PLN: Currency = make_currency!("pln");
/// make_currency!("pln"); // lowercase
/// ```
///
#[macro_export]
macro_rules! make_currency {
    ($cur:literal) => {{
        let bytes = $cur.as_bytes();
        let is_valid = {
            if $cur.len() != 3 || bytes.len() != 3 { false }
            else {
                   (bytes[0] as char).is_ascii_alphabetic() && (bytes[0] as char).is_ascii_uppercase()
                && (bytes[1] as char).is_ascii_alphabetic() && (bytes[1] as char).is_ascii_uppercase()
                && (bytes[2] as char).is_ascii_alphabetic() && (bytes[2] as char).is_ascii_uppercase()
            }
        };
        assert!(is_valid, concat!("Invalid currency \"", $cur, "\" (It should be 3 UPPERCASE english letters)."));

        // We cannot create struct directly there because it has private field.
        // Currency([bytes[0], bytes[1], bytes[2]])
        make_currency($cur)
    }};
    // ($cur:b-literal) => {{ // T O D O: how distinguish byte-literal to have only one macro ??
    //     ...
    // }};
}



#[macro_export]
macro_rules! make_currency_b {
    //($cur:literal) => {{ validate_currency_b!($cur); make_currency_b($cur) }};
    ($cur:literal) => {{
        let bytes = $cur;
        let is_valid = {
            if bytes.len() != 3 { false }
            else {
                   (bytes[2] as char).is_ascii_alphabetic() && (bytes[2] as char).is_ascii_uppercase()
                && (bytes[1] as char).is_ascii_alphabetic() && (bytes[1] as char).is_ascii_uppercase()
                && (bytes[0] as char).is_ascii_alphabetic() && (bytes[0] as char).is_ascii_uppercase()
            }
        };
        assert!(is_valid, concat!("Invalid currency \"", stringify!($cur), "\" (It should be 3 UPPERCASE english letters)."));

        // We cannot create struct directly there because it has private field.
        // Currency([bytes[0], bytes[1], bytes[2]])
        make_currency_b($cur)
    }};
}


pub const USD: Currency = make_currency("USD");
pub const EUR: Currency = make_currency("EUR");





// Tests for private methods/behavior
// Other test are located in ${project}/tests/currency_test.rs
//
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn make_currency2_test() {
        let ascii = b"UAH";
        Currency(*ascii);
        Currency(*ascii);
        Currency(*ascii);
    }

    #[test]
    #[ignore] // It fails because it has access to 'private' (rust-specific behavior, hm...),
              // but it is not critical, you need to do it mutable, watch is not usual case.
    fn impossible_to_change_const_currency_even_for_mutable() {
        let mut temp_obj: Currency = USD;
        temp_obj.0[0] = 'W' as u8;  // Compilation error as expected (impossible to change currency object).
        //assert_eq!(4, internal_adder(2, 2));
        assert_eq!(temp_obj.code_as_string(), "USD");
    }

    #[test]
    #[allow(const_item_mutation)]
    fn impossible_to_change_const_currency_01() {
        USD.0[0] = 'W' as u8;  // Compilation warning 'attempting to modify a `const` item'
        // without modification 'const' object.
        assert_eq!(USD.code_as_string(), "USD");
    }

    // #[test]
    // fn impossible_to_change_const_currency() {
    //     let temp_obj: Currency = USD;
    //     temp_obj.0[0] = 'W' as u8;  // COMPILATION error as expected (impossible to change currency object).
    //     assert_eq!(temp_obj.code_as_string(), "USD");
    // }

    #[test]
    #[allow(unused_mut)]
    fn impossible_to_change_existent_currency_from_outside_package() {

        let mut temp_obj: Currency = USD;
        temp_obj.code_as_ascii_bytes()[0] = 'W' as u8;
        temp_obj.code_as_string().push('Z'); // As expected it does not change currency object.
        println!("{}", temp_obj);

        assert_eq!(temp_obj.code_as_string(), "USD");
    }
}
