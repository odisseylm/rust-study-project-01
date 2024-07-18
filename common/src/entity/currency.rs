use core::char;
use core::str::FromStr;
use sqlx::database::HasValueRef;
use sqlx::error::BoxDynError;
use sqlx_postgres::Postgres;
use crate::json_str_ser_deser_impl;
use crate::string::DisplayValueExample;
//--------------------------------------------------------------------------------------------------



// Ideally it would be nice to use fixedstr::tstr<3> but I do not want to enable
// corresponding not recommended 'fixedstr' feature.
pub type InnerCurStr = fixedstr::str4;


#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, derive_more::Display)]
#[display(fmt = "{}", _0)]
pub struct Currency(InnerCurStr); // ([u8;3]);

json_str_ser_deser_impl! { Currency }

pub type CurrencyFormatError = parse::CurrencyFormatError;

impl Currency {

    pub fn new(currency_code: String) -> Result<Self, CurrencyFormatError> {
        Currency::from_str(currency_code.as_str())
    }

    pub fn code_as_ascii_bytes(&self) -> [u8;3] {
        let s = self.0.as_str().as_bytes();
        let bytes: [u8;3] = [s[0], s[1], s[2]];
        bytes
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()  // unsafe { std::str::from_utf8_unchecked(&self.0) }
    }

    pub fn into_inner(self) -> InnerCurStr {
        self.0
    }
}

impl DisplayValueExample for Currency {
    fn display_value_example() -> &'static str { "AUD" }
}

fn parse_currency(currency_code: & str) -> Result<Currency, CurrencyFormatError> {
    use parse::*;

    if currency_code.is_empty() {
        return Err(CurrencyFormatError::new(ErrorKind::NoCurrency))
    }

    let is_valid: bool = if currency_code.len() != 3 { false } else { is_valid_currency_ascii(currency_code.as_bytes()) };

    if !is_valid {
        Err(CurrencyFormatError::new(ErrorKind::IncorrectCurrencyFormat))
    } else {
        let s = InnerCurStr::try_make(currency_code)
            .map_err(|_|CurrencyFormatError::new(ErrorKind::IncorrectCurrencyFormat)) ?;
        Ok(Currency(s))
    }
}


#[inherent::inherent]
impl FromStr for Currency {
    type Err = CurrencyFormatError;
    #[inline]
    pub fn from_str(s: &str) -> Result<Currency, CurrencyFormatError> {
        parse_currency(s)
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
#[inline(always)]
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
/// use mvv_common::entity::currency::{ Currency, make_currency };
/// const PLN: Currency = make_currency("PLN");
/// assert_eq!(PLN.as_str(), "PLN");
/// assert_eq!(PLN.code_as_ascii_bytes(), *b"PLN");
/// ```
/// ```rust,should_panic
/// use mvv_common::entity::currency::{ Currency, make_currency };
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
    return Currency(InnerCurStr::const_make(currency_code));
}



/// Creates currency.
/// in case of wrong input a panic will be thrown.
///
/// # Examples
/// ```
/// use mvv_common::entity::currency::{ Currency, make_currency_b };
/// const PLN: Currency = make_currency_b(b"PLN");
/// assert_eq!(PLN.as_str(), "PLN");
/// assert_eq!(PLN.code_as_ascii_bytes(), *b"PLN");
/// ```
/// ```rust,should_panic
/// use mvv_common::entity::currency::{ Currency, make_currency_b };
/// // lowercase - error, in case of 'const' there will be compilation error.
/// // The best approach!!! (but not for tests)
/// // const PLN: Currency = make_currency_b(b"pln");
/// make_currency_b(b"pln"); // lowercase
/// ```
///
pub const fn make_currency_b(cur: & 'static [u8;3]) -> Currency {
    let is_valid = is_valid_currency_ascii(cur);
    if !is_valid { const_panic_wrong_currency_ascii(cur) }

    match std::str::from_utf8(cur) {
        Ok(as_str) => Currency(InnerCurStr::const_make(as_str)),
        Err(_) =>
            // It should never happen just there
            const_panic_wrong_currency_ascii(cur)
    }
}



/// Creates currency.
/// in case of wrong input a panic will be thrown.
///
/// # Examples
/// ```
/// use mvv_common::entity::currency::Currency;
/// use mvv_common::make_currency; // macro
/// use mvv_common::entity::currency::make_currency; // required inline function
///
/// const PLN: Currency = make_currency!("PLN");
/// assert_eq!(PLN.as_str(), "PLN");
/// assert_eq!(PLN.code_as_ascii_bytes(), *b"PLN");
/// ```
/// ```rust,should_panic
/// use mvv_common::entity::currency::Currency;
/// use mvv_common::make_currency; // macro
/// use mvv_common::entity::currency::make_currency; // required inline function
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


impl<'r> sqlx::Decode<'r, Postgres> for Currency {
    fn decode(value: <Postgres as HasValueRef<'r>>::ValueRef) -> Result<Self, BoxDynError> {
        let cur = Currency::from_str(value.as_str() ?) ?;
        Ok(cur)
    }
}
impl sqlx::Type<Postgres> for Currency {
    fn type_info() -> <Postgres as sqlx::Database>::TypeInfo {
        <Postgres as sqlx::Database>::TypeInfo::with_name("CHAR") // "CURRENCY")
    }
    fn compatible(ty: &<Postgres as sqlx::Database>::TypeInfo) -> bool {
        <str as sqlx::Type<Postgres>>::compatible(ty)
    }
}




pub mod predefined {
    use super::{ Currency, make_currency };

    pub const USD: Currency = make_currency("USD");
    pub const EUR: Currency = make_currency("EUR");
}


// -------------------------------------------------------------------------------------------------
//                                        Error
// -------------------------------------------------------------------------------------------------


// rust does not support nested structs/types/so on.
// As workaround, I decided to use sub-namespace.
//
pub mod parse {
    use crate::backtrace::BacktraceInfo;

    // #[derive(Debug, PartialEq, Copy, Clone)]
    #[derive(Debug, thiserror::Error, PartialEq)]
    #[derive(Copy, Clone)]
    pub enum ErrorKind {
        #[error("no currency")]
        NoCurrency,
        #[error("Incorrect currency format")]
        IncorrectCurrencyFormat,
    }

    // #[derive(Debug, PartialEq, Copy, Clone)]
    #[derive(thiserror::Error)]
    #[derive(mvv_static_error_macro::MyStaticStructError)]
    pub struct CurrencyFormatError {
        pub kind: ErrorKind,
        // #[source]
        // pub source: ErrorSource,
        pub backtrace: BacktraceInfo,
    }
}



// -------------------------------------------------------------------------------------------------
//                                        Private tests
// -------------------------------------------------------------------------------------------------


// Tests for private methods/behavior
// Other test are located in ${project}/tests/currency_test.rs
//
#[cfg(test)]
mod tests {
    use crate::test::TestSringOps;
    use super::*;
    use super::predefined::*;

    /*
    #[test]
    fn make_currency2_test() {
        let ascii = b"UAH";
        Currency(*ascii);
        Currency(*ascii);
        Currency(*ascii);
    }
    */

    #[test]
    fn make_currency3_test() {
        let ascii = b"UAH";
        make_currency_b(ascii);
        make_currency_b(ascii);
        make_currency_b(ascii);
    }

    /*
    #[test]
    #[ignore] // It fails because it has access to 'private' (rust-specific behavior, hm...),
    // but it is not critical, you need to do it mutable, watch is not usual case.
    fn impossible_to_change_const_currency_even_for_mutable() {
        let mut temp_obj: Currency = USD;
        temp_obj.0[0] = 'W' as u8;  // Compilation error as expected (impossible to change currency object).
        //assert_eq!(4, internal_adder(2, 2));
        // assert_eq!(temp_obj.code_as_string(), "USD");
        assert_eq!(temp_obj.to_test_string(), "USD");
    }

    #[test]
    #[allow(const_item_mutation)]
    fn impossible_to_change_const_currency_01() {
        USD.0[0] = 'W' as u8;  // Compilation warning 'attempting to modify a `const` item'
        // without modification 'const' object.
        // assert_eq!(USD.code_as_string(), "USD");
        assert_eq!(USD.to_test_string(), "USD");
    }
    */

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
        // temp_obj.code_as_string().push('Z'); // As expected it does not change currency object.
        temp_obj.to_test_string().push('Z'); // As expected it does not change currency object.
        println!("{}", temp_obj);

        // assert_eq!(temp_obj.code_as_string(), "USD");
        assert_eq!(temp_obj.to_test_string(), "USD");
    }

    #[test]
    fn debug_test() {
        assert_eq!(make_currency!("USD").to_test_debug_string(), "Currency(USD)");
    }
}
