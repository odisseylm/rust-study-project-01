// #![feature(macro_rules)]

use project01::{ add678, make_currency };
use project01::entities::currency::make_currency;

mod common;

// https://doc.rust-lang.org/book/ch11-03-test-organization.html


#[test]
fn make_currency_fn() {
    common::setup();
    add678(1, 2);
    //add456()
    // assert_eq!(4, adder::add_two(2));

    println!("### make_currency(\"BRL\")");
    make_currency("BRL");
}

#[test]
fn make_currency_macro() {
    common::setup();
    add678(1, 2);
    //add456()
    // assert_eq!(4, adder::add_two(2));

    println!("### make_currency(\"BRL\")");
    //make_currency_macro_temp!("BRL");
    make_currency!("BRL");
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
