// use crate::currency::{Currency, currency_from_bytes, currency_from_string, currency_from_str};

// extern crate core;

// mod main;
// mod currency;
// mod const_examples;
// use currency::USD;
// use crate::currency::{as_printable, as_printable2, Currency, Fuck, PrintableResult };

use project01::entities::{Currency, currency};
use project01::entities::currency::{ CurrencyFormatError, USD };
use project01::util::{as_printable, as_printable_ptr};
use project01::util::result::PrintableResult;

pub fn factorial(n: u128) -> u128 {
    match n {
        0 => 1,
        n => n * factorial(n - 1),
    }
}

fn main() {
    println!("Hello, world!");

    let usd_literal: &str = "USD";
    let _usd_str: String = usd_literal.to_string();
    let _usd_bytes: &[u8] = usd_literal.as_bytes();

    println!("{}", USD);
    //println!("{}", USD.code());
    println!("{} (as code_as_string)", USD.code_as_string());

    // let cur2 = Currency::new(['u' as u8, 'S' as u8, 'D' as u8]);
    let cur2: Result<Currency, CurrencyFormatError> = Currency::new("uSD".to_string());
    //println!("{}", cur2);
    println!("{} (as PrintableResult)", PrintableResult(&cur2));
    println!("{} (as_printable)", as_printable(&cur2));
    println!("{} (as_printable2)", as_printable_ptr(&cur2));

    // let cur2 = Currency::new(['u' as u8, 'S' as u8, 'D' as u8]);
    let cur2: Result<Currency, CurrencyFormatError> = Currency::new("BRL".to_string());
    //println!("{}", cur2);
    println!("{} (as PrintableResult)", PrintableResult(&cur2));
    println!("{} (as_printable)", as_printable(&cur2));
    println!("{} (as_printable2)", as_printable_ptr(&cur2));
    // let currency1: Currency = currency_from_string(&usd_str).expect("Fuck happened.");
    // println!("{}", currency1.to_string());
    //
    // let currency: Currency = currency_from_bytes(usd_bytes).expect("Fuck happened.");
    // println!("{}", currency.to_string());
    //
    // let currency: Currency = currency_from_str("EUR").expect("Fuck happened.");
    // println!("{}", currency.to_string());
}


#[allow(dead_code)]
fn use_dead_code() {
    let usd_code_as_bytes = USD.code_as_ascii_bytes();
    println!("{}", usd_code_as_bytes[0]);

    let usd_code_as_string = currency::EUR.code_as_string();
    println!("{}", usd_code_as_string);

    let usd = Currency::new("USD".to_string());
    println!("{}", usd.unwrap());

    let brl = currency::make_currency("BRL");
    println!("{}", brl);
}
