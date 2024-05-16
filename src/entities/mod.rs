
pub mod currency;

// like export with shorter name
pub use currency::{ Currency, CurrencyFormatError, make_currency, make_currency_b };
pub use amount::{ Amount, amount, ParseAmountError };


pub mod const_examples;
mod macro_samples;
mod amount;
mod test_proc_macro;

