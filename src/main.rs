// use crate::entities::{Currency, currency_from_bytes, currency_from_string, currency_from_str};

//mod main;
mod entities;
mod const_examples;
// mod make_currency;
// use Currency;


fn main() {
    println!("Hello, world!");

    let usd_literal: &str = "USD";
    let _usd_str: String = usd_literal.to_string();
    let _usd_bytes: &[u8] = usd_literal.as_bytes();

    // let currency1: Currency = currency_from_string(&usd_str).expect("Fuck happened.");
    // println!("{}", currency1.to_string());
    //
    // let currency: Currency = currency_from_bytes(usd_bytes).expect("Fuck happened.");
    // println!("{}", currency.to_string());
    //
    // let currency: Currency = currency_from_str("EUR").expect("Fuck happened.");
    // println!("{}", currency.to_string());
}
