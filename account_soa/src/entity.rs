// like export with shorter path
// pub use currency::{ Currency, CurrencyFormatError, make_currency, make_currency_b };
// pub use amount::{ Amount, amount, AmountFormatError };


pub mod account;
pub mod user;
mod investigation;

// Experimental approach... What is better??
pub mod prelude {
    pub use super::user::UserId;
    pub use super::account::AccountId;
    pub use super::account::Account;
    pub type Currency = mvv_common::entity::Currency;
    pub type CurrencyFormatError = mvv_common::entity::CurrencyFormatError;
    pub type Amount = mvv_common::entity::Amount;
    pub type AmountFormatError = mvv_common::entity::AmountFormatError;
}

// Experimental approach... What is better??
pub mod entity {
    pub use super::user::UserId;
    pub use super::account::AccountId;
    pub use super::account::Account;
}
