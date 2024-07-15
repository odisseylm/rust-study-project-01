
pub mod currency;

// like export with shorter path
// pub use currency::{ Currency, CurrencyFormatError, make_currency, make_currency_b };
// pub use amount::{ Amount, amount, ParseAmountError };


pub mod amount;


pub mod account;
pub mod id;
pub mod user;
mod investigation;

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


pub mod bd {
    use core::fmt::{ self, Formatter };
    use bigdecimal::BigDecimal;

    // Default BigDecimal Debug impl shows very unfriendly info
    pub fn bd_dbg_fmt(bd: &BigDecimal, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{bd} ({bd:?})")
    }
}