// #![feature(error_generic_member_access)]
// #![feature(trait_alias)]
// #![feature(type_alias_impl_trait)]
// #![feature(lazy_cell)]
// #![feature(lint_reasons)]
// #![feature(error_generic_member_access)]
// #![feature(provide_any)]
// #![feature(let_chains)]

pub mod entities;
pub mod util;
pub mod adder;
pub mod web_server;
pub mod service;
pub mod database;
pub mod rest;
pub mod web;
pub mod json;

// by full path
// use crate::entities::currency::{ Currency, make_currency };
// by shorter path
// use crate::entities::{ Currency, make_currency };
// use crate::entities::currency;

pub fn add678(v1: i32, v2: i32) -> i32 { v1 + v2 }


/*
#[allow(dead_code)]
fn just_usage() {
    let usd_code_as_bytes = currency::USD.code_as_ascii_bytes();
    println!("{}", usd_code_as_bytes[0]);

    let usd_code_as_string = currency::EUR.code_as_string();
    println!("{}", usd_code_as_string);

    let usd = Currency::new("USD".to_string());
    println!("{}", usd.u n w r a p());

    let brl = make_currency("BRL");
    println!("{}", brl);

    let added = adder::add852(1, 2);
    println!("{}", added);
}
*/


//mod adder;

// Define this in a crate called `adder`.
// pub fn add(a: i32, b: i32) -> i32 {
//     a + b
// }


// #[cfg(test)]
// mod tests {
//     use ctor::ctor;
//
//     // Seems it does not work.
//     #[ctor]
//     fn init_color_backtrace() {
//         println!("### init_color_backtrace");
//         color_backtrace::install();
//     }
// }
