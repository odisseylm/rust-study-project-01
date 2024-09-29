// like export with shorter path
// pub use currency::{ Currency, CurrencyFormatError, make_currency, make_currency_b };
// pub use amount::{ Amount, amount, AmountFormatError };


pub mod account;
pub mod user;
mod investigation;
mod iban;
mod id;

// Experimental approach... What is better??
pub mod prelude {
    pub use super::user::UserId;
    pub use super::account::AccountId;
    pub use super::account::Account;
    pub type Currency = mvv_common_bank_entities::Currency;
    pub type CurrencyFormatError = mvv_common_bank_entities::CurrencyFormatError;
    pub type Amount = mvv_common_bank_entities::Amount;
    pub type AmountFormatError = mvv_common_bank_entities::AmountFormatError;
}

// Experimental approach... What is better??
// pub mod entity {
//     pub use super::user::UserId;
//     pub use super::account::AccountId;
//     pub use super::account::Account;
// }

pub use account::{ AccountId, Account, };
pub use id::ClientId;
pub use iban::IbanWrapper;
pub use iban::IbanRefWrapper;
