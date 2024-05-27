use std::str::{ FromStr };
use bigdecimal::{ BigDecimal, BigDecimalRef, ParseBigDecimalError };
use crate::entities::currency::Currency;
// use crate::entities::currency::Currency;       // ++
// use ::project01::entities::currency::Currency; // --
// use project01::entities::currency::Currency;   // --
// use self::super::currency::Currency;           // ++
// use super::currency::Currency;                 // ++
use crate::util::UncheckedResultUnwrap;


// #[derive(Debug)]
#[derive(PartialEq, Eq)]
pub struct Amount {
    value: BigDecimal,
    currency: Currency,
}

impl core::fmt::Debug for Amount {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Amount {{ {} {} ({:?}) }}", self.value, self.currency, self.value)
    }
}

pub mod parse_amount {
    pub type ParseAmountError = crate::entities::parse_amount::ParseAmountError;
    pub type ErrorKind = crate::entities::parse_amount::ErrorKind;
    pub type ErrorSource = crate::entities::parse_amount::ErrorSource;
}

pub mod ops {
    pub type AmountOpsError = crate::entities::amount_ops::ops::AmountOpsError;
    pub type ErrorKind = crate::entities::amount_ops::ops::ErrorKind;
    pub type ErrorSource = crate::entities::amount_ops::ops::ErrorSource;
}


impl core::fmt::Display for Amount {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.value, self.currency)
    }
}


impl Amount {

    pub fn with_str_amount(amount: &str, currency: Currency) -> Result<Self, ParseBigDecimalError> {
        let bd: Result<BigDecimal, ParseBigDecimalError> = BigDecimal::from_str(amount);
        return bd.map(|am| Amount { value: am, currency } );
    }

    pub fn with_str_amount_unchecked(amount: &str, currency: Currency) -> Self {
        Amount::with_str_amount(amount, currency).unchecked_unwrap()
    }

    #[inline]
    pub fn new(amount: BigDecimal, currency: Currency) -> Amount {
        Amount { value: amount, currency }
    }

    // fn from_string(amount_with_currency: &str)

    pub fn currency(&self) -> Currency {
        self.currency
    }

    // I do not see sense to have it since copy of currency is cheap.
    // pub fn currency_ref(&self) -> &Currency { &self.currency }

    pub fn value_ref(&self) -> &BigDecimal {
        &self.value
    }

    pub fn value_bd_ref(&self) -> BigDecimalRef<'_> {
        self.value.to_ref()
    }

    pub fn with_value(&self, amount: BigDecimal) -> Amount {
        Amount { value: amount, currency: self.currency }
    }

    // I do not see sense to have function to 'change' currency (there are no such user/bank cases).
    // Having such method just can provoke making incorrect/senseless operations.
    // pub fn with_currency(&self, currency: Currency) -> Amount {
    //     Amount { value: self.value.clone(), currency }
    // }
}

// Just short alias (similar to kotlin style)
#[inline]
pub fn amount(amount: BigDecimal, currency: Currency) -> Amount { Amount::new(amount, currency) }



impl FromStr for Amount {
    type Err = parse_amount::ParseAmountError;

    #[inline]
    fn from_str(s: &str) -> Result<Amount, Self::Err> { crate::entities::parse_amount::parse_amount(s) }
}
