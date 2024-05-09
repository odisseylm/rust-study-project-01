
pub mod entities;
pub mod util;
mod adder;

// by full path
// use crate::entities::currency::{ Currency, make_currency };
// by shorter path
use crate::entities::{ Currency, make_currency };
use crate::entities::currency;

pub fn add678(v1: i32, v2: i32) -> i32 { v1 + v2 }


#[allow(dead_code)]
fn just_usage() {
    let usd_code_as_bytes = currency::USD.code_as_ascii_bytes();
    println!("{}", usd_code_as_bytes[0]);

    let usd_code_as_string = currency::EUR.code_as_string();
    println!("{}", usd_code_as_string);

    let usd = Currency::new("USD".to_string());
    println!("{}", usd.unwrap());

    let brl = make_currency("BRL");
    println!("{}", brl);

    let added = adder::add852(1, 2);
    println!("{}", added);
}


//mod adder;

// Define this in a crate called `adder`.
// pub fn add(a: i32, b: i32) -> i32 {
//     a + b
// }
