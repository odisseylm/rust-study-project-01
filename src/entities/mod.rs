
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
pub mod user;

// Experimental approach... What is better??
pub mod prelude {
    pub use super::id::Id;
    pub use super::user::UserId;
    pub use super::account::AccountId;

    pub use super::currency::Currency;
    pub use super::currency::parse::CurrencyFormatError;
    pub use super::amount::Amount;
    pub use super::amount::parse::ParseAmountError as AmountFormatError;
    pub use super::account::Account;
}

// Experimental approach... What is better??
pub mod entity {
    pub use super::id::Id;
    pub use super::user::UserId;
    pub use super::account::AccountId;

    pub use super::currency::Currency;
    pub use super::currency::parse::CurrencyFormatError;
    pub use super::amount::Amount;
    pub use super::amount::parse::ParseAmountError as AmountFormatError;
    pub use super::account::Account;
}
