// use crate::common;

// extern crate mockall;

// use crate::common;
// use crate::common;
//
// use currency::USD;
// use crate::currency::{as_printable, as_printable2, Currency, Fuck, PrintableResult};

use project01::add678;
use project01::entities::currency::make_currency;

mod common;

// bla-bla

// https://doc.rust-lang.org/book/ch11-03-test-organization.html

// use crate::
// use crate::entities::currency::Currency;
// use crate::entities::currency::make_currency;


#[test]
fn it_adds_two() {
    common::setup();
    add678(1, 2);
    //add456()
    // assert_eq!(4, adder::add_two(2));

    println!("### make_currency(\"BRL\")");
    make_currency("BRL");
}

#[test]
#[should_panic(expected = "Invalid currency (It should be 3 UPPERCASE english letters).")]
fn some_testing_failure() {
    common::setup();

    add678(1, 2);
    //add456()
    // assert_eq!(4, adder::add_two(2));

    //make_currency("BRL");
    make_currency("BR");
}
