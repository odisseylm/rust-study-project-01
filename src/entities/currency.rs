
// #![macro_use]

// mod entities;

use core::fmt;
// use std::fmt::Display;
use std::char;

//extern crate static_assertions as sa;


#[derive(Debug)]
// #[derive(core::fmt::Display)]
// pub struct Currency(pub String);
pub struct Currency([u8;3]);

impl Currency {
    /*
    pub fn new_from_bytes(currency_code: &[u8;3]) -> Result<Self, Fuck> {
        let is_valid = is_validate_currency_code_as_ascii_bytes(&currency_code);
        if !is_valid { Err(Fuck{}) }
        else { Ok(Currency(*currency_code)) }
    }
    */
    pub fn new(currency_code: String) -> Result<Self, Fuck> {
        let is_valid = is_validate_currency_code_string(&currency_code);

        if !is_valid { Err(Fuck{}) }
        else {
            let as_bytes = currency_code.as_bytes();
            Ok(Self([as_bytes[0], as_bytes[1], as_bytes[2]]))
        }
    }
    pub fn code_as_ascii_bytes(&self) -> [u8;3] {
        self.0
    }
    pub fn code_as_string(&self) -> String {
        // a bit overcomplicated...
        //String::from_utf8(Vec::from(self.0)).unwrap()

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

#[derive(Debug)]
pub struct Fuck {} // TODO: use something rust-standard

impl fmt::Display for Fuck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fuck")
    }
}

// impl fmt::Display for Result<Currency, Fuck> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         write!(f, "{}{}{}", self.0[0] as char, self.0[1] as char, self.0[2] as char)
//     }
// }


pub struct PrintableResult<'a, T: fmt::Display, E: fmt::Display>(pub &'a Result<T, E>);

#[inline]
// fn as_printable<'a, T: fmt::Display, E: fmt::Display>(r: &'a Result<T, E>) -> PrintableResult<'a, T, E> {
pub fn as_printable<T: fmt::Display, E: fmt::Display>(r: &Result<T, E>) -> PrintableResult<T, E> {
    return PrintableResult(r);
}

impl<'a, T: fmt::Display, E: fmt::Display> fmt::Display for PrintableResult<'a, T, E> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            Ok(ok)   => { write!(f, "{}", ok)  }
            Err(err) => { write!(f, "{}", err) }
        }
    }
}

#[inline]
pub fn as_printable2<'a, T: fmt::Display, E: fmt::Display>(r: &'a Result<T, E>) -> & 'a dyn fmt::Display {
    return match r {
        Ok(ok)   => { ok  }
        Err(err) => { err }
    }
}

// fn panic_wrong_currency(currency: &String) -> ! {
//     panic!("Invalid currency '{}' (It should be 3 UPPERCASE english letters.).", currency)
// }
// fn panic_wrong_currency_bytes(currency: &[u8]) -> ! {
//     // let mut s: String = String::with_capacity(currency.len());
//     // for b in currency.iter() { s.push(*b as char) }
//     // // currency.iter().for_each(|b| s.push(*b as char) );
//
//     let as_str: String = currency.iter().map(|b| *b as char).collect::<String>();
//     panic!("Invalid currency '{}' (It should be 3 UPPERCASE english letters.).", as_str);
// }


// #[allow(unused_variables)]
const fn const_panic_wrong_currency(_currency: & 'static str) -> ! {
    // TODO: print 'currency' in some way
    // const MSG: &str = const_format::concatcp!!(2u8, "+", 2u8, '=', 2u8 + 2);
    // const MSG: &str = const_format::concatcp!!("Invalid currency [", c1, c2, c3, "].");
    // panic!("Invalid currency {currency}", currency = _currency);
    // concat!(1, 2, 3, "abc")
    // panic!(concat!("Invalid currency (It should be 3 UPPERCASE english letters).", _currency)); // not compiled
    panic!("Invalid currency (It should be 3 UPPERCASE english letters).")
}
const fn const_panic_wrong_currency_bytes(_currency: & 'static [u8]) -> ! {
    // TODO: print 'currency' in some way
    panic!("Invalid currency (It should be 3 UPPERCASE english letters).")
}

const fn is_valid_currency_char(ch: char) -> bool {
    ch.is_ascii_alphabetic() && ch.is_ascii_uppercase()
}
const fn is_valid_currency_char_byte(ch: u8) -> bool {
    is_valid_currency_char(ch as char)
}

fn is_validate_currency_code_string(cur: &String) -> bool {
    if cur.len() != 3 { return false; }
    let bytes = cur.as_bytes();
    return is_validate_currency_code_as_ascii_bytes(bytes);
}

// hm... like kotlin inline dependent functions, this validate_currency should be also published.
pub const fn is_validate_currency_code_literal(cur: & 'static str) -> bool {
    if cur.len() != 3 { return false; }
    let bytes = cur.as_bytes();
    return is_validate_currency_code_as_ascii_bytes(bytes);
    // if bytes.len() != 3 { return false; }
    //
    // let valid: bool = is_valid_currency_char_byte(bytes[0])
    //     && is_valid_currency_char_byte(bytes[1])
    //     && is_valid_currency_char_byte(bytes[2]);
    // return valid
}

// hm... like kotlin inline dependent functions, this validate_currency should be also published.
pub const fn is_validate_currency_code_as_ascii_bytes(cur: &[u8]) -> bool {
    if cur.len() != 3 { return false; }
    let bytes = cur; //.as_bytes();
    if bytes.len() != 3 { return false; }

    let valid: bool = is_valid_currency_char_byte(bytes[0])
        && is_valid_currency_char_byte(bytes[1])
        && is_valid_currency_char_byte(bytes[2]);
    return valid
}

/*
const fn validate_currency_code(cur: & 'static str) {
    if cur.len() != 3 { panic_wrong_currency(cur) }
    let bytes = cur.as_bytes();
    if bytes.len() != 3 { panic_wrong_currency(cur) }

    let valid: bool = is_valid_currency_char_byte(bytes[0])
        && is_valid_currency_char_byte(bytes[1])
        && is_valid_currency_char_byte(bytes[2]);
    if !valid { panic_wrong_currency(cur) }
}
*/

/// Creates currency.
/// in case of wrong input a panic will be thrown.
///
/// # Examples
/// ```
/// use project01::entities::make_currency;
/// let result = make_currency("PLN");
/// assert_eq!(result.code_as_string(), "PLN");
/// assert_eq!(result.code_as_ascii_bytes(), *b"PLN");
/// ```
/// ```rust,should_panic
/// use project01::entities::make_currency;
/// make_currency("pln"); // lowercase
/// ```
pub const fn make_currency(cur: & 'static str) -> Currency {
    //validate_currency_code(cur);
    let is_valid = is_validate_currency_code_literal(cur);
    if !is_valid { const_panic_wrong_currency(cur) }
    let bytes = cur.as_bytes();
    return Currency([bytes[0], bytes[1], bytes[2]]);
}

/// Creates currency.
/// in case of wrong input a panic will be thrown.
///
/// # Examples
/// ```
/// use project01::entities::make_currency_b;
/// let result = make_currency_b(b"PLN");
/// assert_eq!(result.code_as_string(), "PLN");
/// assert_eq!(result.code_as_ascii_bytes(), *b"PLN");
/// ```
/// ```rust,should_panic
/// use project01::entities::make_currency_b;
/// make_currency_b(b"pln"); // lowercase
/// ```
pub const fn make_currency_b(cur: & 'static [u8;3]) -> Currency { // TODO: rename
    //validate_currency_code(cur);
    let is_valid = is_validate_currency_code_as_ascii_bytes(cur);
    if !is_valid { const_panic_wrong_currency_bytes(cur) }
    return Currency([cur[0], cur[1], cur[2]]);
}


#[allow(unused_macros)]
macro_rules! say_hello {
    () => (
        println!("### Hello, world!");
    );
}
#[allow(unused_macros)]
macro_rules! create_function {
    ($func_name:ident) => (
        fn $func_name() {
            println!("You called {:?}()", stringify!($func_name));
        }
    );
}

#[allow(unused_macros)]
macro_rules! assert_equal_len {
    ($a:expr, $b:expr, $func:ident, $op:tt) => {
        assert!($a.len() == $b.len(),
                "{:?}: dimension mismatch: {:?} {:?} {:?}",
                stringify!($func),
                ($a.len(),),
                stringify!($op),
                ($b.len(),));
    };
}

#[allow(unused_macros)]
macro_rules! do_thing {
    (print $metavar:literal) => {
        println!("{}", $metavar)
    };
}
// do_thing!(print 3);  => println!("{}", 3);

/// Creates currency.
/// in case of wrong input a panic will be thrown.
///
/// # Examples
/// ```
/// use project01::make_currency;
/// use project01::entities::currency::make_currency;
///
/// let result = make_currency!("PLN");
/// assert_eq!(result.code_as_string(), "PLN");
/// assert_eq!(result.code_as_ascii_bytes(), *b"PLN");
/// ```
/// ```rust,should_panic
/// use project01::make_currency;
/// use project01::entities::currency::make_currency;
///
/// make_currency!("pln"); // lowercase
/// ```
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
        assert!(is_valid, "Invalid currency (It should be 3 UPPERCASE english letters).");
        // We cannot create struct directly there because it has private field.
        // Currency([bytes[0], bytes[1], bytes[2]])
        make_currency($cur)
    }};
    // ($cur:b-literal) => {{ // T O D O: how distinguish byte-literal
    //     validate_currency_b!($cur); make_currency_b($cur)
    // }};
}
#[macro_export] // hm... like kotlin inline dependent functions, this validate_currency should be also published.
macro_rules! validate_currency {
    ($cur:literal) => {
        // assert!(is_validate_currency_code_literal($cur), "Invalid currency (It should be 3 UPPERCASE english letters).");
        // full path to avoid manual import later.
        assert!(is_validate_currency_code_literal($cur), "Invalid currency (It should be 3 UPPERCASE english letters).");
    }
    // ($cur:expr) => {
    //     assert!(is_validate_currency_code_as_ascii_bytes($cur), "Invalid currency (It should be 3 UPPERCASE english letters).");
    // }
}
// #[macro_export]
// macro_rules! make_currency_macro_temp {
//     ($cur:literal) => { make_currency($cur) }
// }

#[macro_export] // hm... like kotlin inline dependent functions, this validate_currency should be also published.
macro_rules! validate_currency_b {
    ($cur:literal) => {
        assert!(is_validate_currency_code_as_ascii_bytes($cur), "Invalid currency (It should be 3 UPPERCASE english letters).");
    }
}
#[macro_export]
macro_rules! make_currency_b {
    ($cur:literal) => {{ validate_currency_b!($cur); make_currency_b($cur) }};
}

#[allow(unused_macros)]
macro_rules! foo {
    (_ bool) => {
        println!("got bool");
    };
    (_ Result<i32>) => {
        println!("got Result<i32>");
    };
    (_ $tp:ty) => {
        println!("fallback to type: {}", stringify!($tp));
    };
    // ($($tp:tt)*) => {
    //     foo!(_ $($tp)*);
    // };
}

#[allow(unused_macros)]
macro_rules! foo2 {
    ($tp:ty) => {
        foo!(_ $tp);
    };
    (_ bool) => {
        println!("got bool");
    };
    (_ Result<i32>) => {
        println!("got Result<i32>");
    };
    (_ $tp:ty) => {
        println!("fallback to type: {}", stringify!($tp));
    };

}

// const fn currency_from_chars(c1: char, c2: char, c3: char) -> Currency {
//     let valid: bool = is_valid_currency_char(c1) && is_valid_currency_char(c2) && is_valid_currency_char(c3);
//     if !valid { panic_wrong_currency() }
//     return Currency([c1 as u8, c2 as u8, c3 as u8]);
// }


pub const USD: Currency = make_currency("USD");
pub const EUR: Currency = make_currency("EUR");



// struct Amount {
//     value: f32,
//     currency: Currency,
// }



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_temp() {
        // say_hello!();
        // create_function!(rrr);
        // rrr();

        // fn local_fn() { println!("### local_fn") }
        // assert_equal_len!("a", "aa", local_fn, +);
        // assert_equal_len!("a", "a", local_fn, +);

        // foo!(true bool);
        foo!(_ bool);
        foo2!(_ bool);
        //foo!(bool);
        foo2!(bool);

        let cur: Currency = make_currency!("USD");
        assert_eq!(cur.code_as_string(), "USD");

        // let cur: Currency = make_currency2!("US");
        // assert_eq!(cur.code_as_string(), "US");

        // const CUR43: Currency = make_currency!("usd"); // fails at compile time  - very-very GOOD
        // assert_eq!(CUR43.code_as_string(), "USD");

        // const CUR43: Currency = make_currency!("US");  // fails at compile time  - very-very GOOD
        // assert_eq!(CUR43.code_as_string(), "USD");

        let cur43: Currency = make_currency!("usd"); // Does NOT fail at compile time (only at runtime) - BAD!!! Why??
        assert_eq!(cur43.code_as_string(), "USD");

        //const CUR44: Currency = make_currency2!("US");
        //assert_eq!(CUR44.code_as_string(), "US");

        // const CUR44: Currency = make_currency3!("US");
        // assert_eq!(CUR44.code_as_string(), "US");

        let cur45_4: Currency = make_currency!("usd");
        assert_eq!(cur45_4.code_as_string(), "US");

        // noinspection RsConstNaming
        // const cur45_2: Currency = make_currency("US"); // fails at compile time - vey good
        // assert_eq!(cur45_2.code_as_string(), "US");

        let not_direct_literal = "US";
        let cur45_0: Currency = make_currency(not_direct_literal);
        // let cur45: Currency = make_currency3!(not_direct_literal);
        assert_eq!(cur45_0.code_as_string(), "US");

        let cur46: Currency = make_currency!("US");
        assert_eq!(cur46.code_as_string(), "US");
    }

    // ??? Does not work !!!
    //#[setup]
    // pub fn setup() {
    //     // setup code specific to your library's tests would go here
    //     println!("### setup()");
    // }

    #[test]
    fn make_currency_test() {
        const UAH: Currency = make_currency("UAH");
        assert_eq!(UAH.code_as_string(), "UAH");
        assert_eq!(UAH.code_as_ascii_bytes(), *b"UAH");
        assert_eq!(UAH.code_as_ascii_bytes(), ['U' as u8, 'A' as u8, 'H' as u8]);

        let jpy = make_currency("JPY");
        assert_eq!(jpy.code_as_string(), "JPY");
        assert_eq!(jpy.code_as_ascii_bytes(), *b"JPY");
        assert_eq!(jpy.code_as_ascii_bytes(), ['J' as u8, 'P' as u8, 'Y' as u8]);
    }

    #[test]
    fn make_currency_b_test() {
        const UAH: Currency = make_currency_b(b"UAH");
        assert_eq!(UAH.code_as_string(), "UAH");
        assert_eq!(UAH.code_as_ascii_bytes(), *b"UAH");
        assert_eq!(UAH.code_as_ascii_bytes(), ['U' as u8, 'A' as u8, 'H' as u8]);

        //const UAH2: Currency = make_currency!(b"UAH");
        const UAH2: Currency = make_currency_b!(b"UAH");
        assert_eq!(UAH2.code_as_string(), "UAH");
        assert_eq!(UAH2.code_as_ascii_bytes(), *b"UAH");
        assert_eq!(UAH2.code_as_ascii_bytes(), ['U' as u8, 'A' as u8, 'H' as u8]);

        let jpy = make_currency_b(b"JPY");
        assert_eq!(jpy.code_as_string(), "JPY");
        assert_eq!(jpy.code_as_ascii_bytes(), *b"JPY");
    }

    #[test]
    fn make_currency2_test() {
        let aaa = b"UAH";
        Currency(*aaa);
        Currency(*aaa);
        Currency(*aaa);

        const UAH: Currency = make_currency("UAH");
        assert_eq!(UAH.code_as_string(), "UAH");
        assert_eq!(UAH.code_as_ascii_bytes(), ['U' as u8, 'A' as u8, 'H' as u8]);

        let jpy = make_currency("JPY");
        assert_eq!(jpy.code_as_string(), "JPY");
        assert_eq!(jpy.code_as_ascii_bytes(), ['J' as u8, 'P' as u8, 'Y' as u8]);
    }

    #[test]
    fn currency_new_test() {
        let uah = Currency::new("UAH".to_string()).unwrap();
        assert_eq!(uah.code_as_string(), "UAH");
        assert_eq!(uah.code_as_ascii_bytes(), ['U' as u8, 'A' as u8, 'H' as u8]);
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: Fuck")]
    fn currency_new_with_wrong() {
        Currency::new("uAH".to_string()).unwrap();
    }

    #[test]
    #[should_panic(expected = "Invalid currency (It should be 3 UPPERCASE english letters).")]
    fn currency_new_macro_with_wrong() {
        make_currency!("uAH");
    }

    #[test]
    fn currency_new_with_wrong_02() {
        let cur = Currency::new("uAH".to_string());
        assert!(cur.is_err());
    }

    #[test]
    #[should_panic] // just example without message
    fn impossible_make_wrong_const_literal_currency_for_non_alpha() {
        // As expected, due to 'const' var qualifier we have compilation error
        //const cur1: Currency = make_currency("1US");
        make_currency("1US");
    }

    #[test]
    #[should_panic] // just example without message
    fn macro_impossible_make_wrong_const_literal_currency_for_non_alpha() {
        // As expected, due to 'const' var qualifier we have compilation error
        //const cur1: Currency = make_currency("1US");
        make_currency!("1US");
    }

    #[test]
    #[should_panic(expected = "Invalid currency (It should be 3 UPPERCASE english letters).")]
    fn impossible_make_wrong_const_literal_currency_for_wrong_length() {
        make_currency("US");
        make_currency("USDD");
    }

    #[test]
    #[should_panic(expected = "Invalid currency (It should be 3 UPPERCASE english letters).")]
    fn macro_impossible_make_wrong_const_literal_currency_for_wrong_length() {
        make_currency!("US");
        make_currency!("USDD");
    }

    #[test]
    fn impossible_make_wrong_const_literal_currency_for_non_lowercase() {
        // make_currency("usd");
        let result = std::panic::catch_unwind(|| make_currency("usd"));
        assert!(result.is_err());
        // T O D O: how to get error message???
        //assert_eq!(result.unwrap_err(), "dsd");
    }

    /*
    // see https://docs.rs/test-case/3.3.1/test_case/index.html#
    #[test_case::test_case(-2, -4 ; "when both operands are negative")]
    #[test_case::test_case(2,  4  ; "when both operands are positive")]
    #[test_case::test_case(4,  2  ; "when operands are swapped")]
    fn multiplication_tests(x: i8, y: i8) {
        let actual = (x * y).abs();
        assert_eq!(8, actual)
    }

    // see https://docs.rs/test-case/3.3.1/test_case/index.html#
    #[test_case::test_case(0 => panics)]
    #[test_case::test_case(1)]
    fn test_divisor(divisor: usize) {
        let _result = 1 / divisor;
    }
    */

    // #[test]
    // fn impossible_make_wrong_const_literal_currency__non_lowercase_2() {
    //     // fluent-asserter library, version 0.1.7
    //     // https://github.com/dmoka/fluent-asserter
    //     // Seems it is not developing now :-(
    //     //
    //     assert_that_code!(|| make_currency("usd"))
    //         .panics()
    //         .with_message("some panic message");
    // }

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
    #[ignore] // It fails because it has access to 'private' (rust-specific behavior, hm...),
              // but it is not critical, you need to do it mutable, watch is not usual case.
    fn impossible_to_change_const_currency_even_for_mutable() {
        let mut temp_obj: Currency = USD;
        temp_obj.0[0] = 'W' as u8;  // Compilation error as expected (impossible to change currency object).
        //assert_eq!(4, internal_adder(2, 2));
        assert_eq!(temp_obj.code_as_string(), "USD");
    }

    #[test]
    #[allow(unused_mut)]
    fn impossible_to_change_existent_currency_from_outside_package() {

        let mut temp_obj: Currency = USD;
        temp_obj.code_as_ascii_bytes()[0] = 'W' as u8;
        temp_obj.code_as_string().push('Z'); // As expected it does not change currency object.
        println!("{}", temp_obj);

        assert_eq!(temp_obj.code_as_string(), "USD");
    }

    #[test]
    fn use_public_constants() {
        assert_eq!(USD.code_as_string(), "USD");
        assert_eq!(EUR.code_as_ascii_bytes()[0], 'E' as u8);
        assert_eq!(EUR.code_as_ascii_bytes()[1], 'U' as u8);
        assert_eq!(EUR.code_as_ascii_bytes()[2], 'R' as u8);
    }
}
