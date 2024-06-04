
pub mod currency;

// like export with shorter path
// pub use currency::{ Currency, CurrencyFormatError, make_currency, make_currency_b };
// pub use amount::{ Amount, amount, ParseAmountError };


pub mod const_examples;
pub mod amount;


mod macro_samples;
mod test_proc_macro;
mod amount_parse_old;
pub mod parse_amount;
pub mod amount_ops;
mod type_path_usage;
pub mod account;
pub mod id;
pub mod serde_json_bd;


// pub use crate::parse_amount::{ ParseAmountError as amount::parse_amount::ParseAmountError };
// pub use parse_amount::{ ParseAmountError as amount222 };
