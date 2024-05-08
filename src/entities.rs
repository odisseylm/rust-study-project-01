// mod entities;

// use std::fmt::Display;
use std::char;

//extern crate static_assertions as sa;


#[derive(Debug)]
// pub struct Currency(pub String);
pub struct Currency(pub [u8;3]);

#[derive(Debug)]
pub struct Fuck {} // TODO: use something rust-standard


const fn is_valid_currency_char(ch: char) -> bool {
    ch.is_ascii_alphabetic() && ch.is_ascii_uppercase()
}
const fn is_valid_currency_char_byte(ch: u8) -> bool {
    is_valid_currency_char(ch as char)
}

// #[allow(unused_variables)]
const fn panic_wrong_currency(_currency: & 'static str) -> ! {
    // TODO: print 'currency' in some way
    // const MSG: &str = const_format::concatcp!!(2u8, "+", 2u8, '=', 2u8 + 2);
    // const MSG: &str = const_format::concatcp!!("Invalid currency [", c1, c2, c3, "].");
    // panic!(cf:: const_format!("Invalid currency {c1}{c2}{c3}"))
    panic!("Invalid currency (It should be 3 UPPERCASE english letters.).")
}

const fn make_currency(cur: & 'static str) -> Currency {
    if cur.len() != 3 { panic_wrong_currency(cur) }
    let bytes = cur.as_bytes();
    if bytes.len() != 3 { panic_wrong_currency(cur) }

    let valid: bool = is_valid_currency_char_byte(bytes[0])
        && is_valid_currency_char_byte(bytes[1])
        && is_valid_currency_char_byte(bytes[2]);
    if !valid { panic_wrong_currency(cur) }
    return Currency([bytes[0], bytes[1], bytes[2]]);
}

// const fn currency_from_chars(c1: char, c2: char, c3: char) -> Currency {
//     let valid: bool = is_valid_currency_char(c1) && is_valid_currency_char(c2) && is_valid_currency_char(c3);
//     if !valid { panic_wrong_currency() }
//     return Currency([c1 as u8, c2 as u8, c3 as u8]);
// }


pub const USD: Currency = make_currency("USD");
pub const EUR: Currency = make_currency("EUR");
