use bigdecimal::{BigDecimal, Zero};
use crate::entity::amount::{ Amount, amount };
use crate::backtrace2::BacktraceCell;
use crate::entity::currency::Currency;
//--------------------------------------------------------------------------------------------------



/*
impl PartialEq for Amount {
    fn eq(&self, other: &Self) -> bool {
        self.currency() == other.currency() && self.value_ref() == other.value_ref()
    }
    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}
impl Eq for Amount { }
*/


impl core::ops::Add<Amount> for Amount {
    type Output = Result<Amount, AmountOpsError>;
    fn add(self, rhs: Amount) -> Self::Output {
        if self.currency() != rhs.currency() {
            Err(AmountOpsError::new(ErrorKind::DifferentCurrencies(self.currency(), rhs.currency())))
        } else {
            Ok(amount(self.value_ref().add(rhs.value_ref()), self.currency()))
        }
    }
}
impl<'a> core::ops::Add<& 'a Amount> for &Amount {
    type Output = Result<Amount, AmountOpsError>;
    fn add(self, rhs: & 'a Amount) -> Self::Output {
        if self.currency() != rhs.currency() {
            Err(AmountOpsError::new(ErrorKind::DifferentCurrencies(self.currency(), rhs.currency())))
        } else {
            Ok(amount((&self.value_ref()).add((&rhs).value_ref()), self.currency()))
        }
    }
}

impl core::ops::Sub<Amount> for Amount {
    type Output = Result<Amount, AmountOpsError>;
    fn sub(self, rhs: Amount) -> Self::Output {
        if self.currency() != rhs.currency() {
            Err(AmountOpsError::new(ErrorKind::DifferentCurrencies(self.currency(), rhs.currency())))
        } else {
            Ok(amount(self.value_ref().sub(rhs.value_ref()), self.currency()))
        }
    }
}
impl<'a> core::ops::Sub<& 'a Amount> for &Amount {
    type Output = Result<Amount, AmountOpsError>;
    fn sub(self, rhs: & 'a Amount) -> Self::Output {
        if self.currency() != rhs.currency() {
            Err(AmountOpsError::new(ErrorKind::DifferentCurrencies(self.currency(), rhs.currency())))
        } else {
            Ok(amount((&self.value_ref()).sub((&rhs).value_ref()), self.currency()))
        }
    }
}

impl core::ops::Mul<BigDecimal> for Amount {
    type Output = Amount;
    fn mul(self, rhs: BigDecimal) -> Self::Output {
        amount(self.value_ref().mul(rhs), self.currency())
    }
}
impl<'a> core::ops::Mul<& 'a BigDecimal> for &Amount {
    type Output = Amount;
    fn mul(self, rhs: & 'a BigDecimal) -> Self::Output {
        amount((&self.value_ref()).mul(rhs), self.currency())
    }
}

impl core::ops::Mul<Amount> for BigDecimal {
    type Output = Amount;
    fn mul(self, rhs: Amount) -> Self::Output {
        amount(rhs.value_ref().mul(self), rhs.currency())
    }
}
impl<'a> core::ops::Mul<& 'a Amount> for & 'a BigDecimal {
    type Output = Amount;
    fn mul(self, rhs: & 'a Amount) -> Amount {
        amount((&rhs.value_ref()).mul(self), rhs.currency())
    }
}

impl core::ops::Div<BigDecimal> for Amount {
    type Output = Result<Amount, AmountOpsError>;
    fn div(self, rhs: BigDecimal) -> Self::Output {
        if rhs.is_zero() {
            Err(AmountOpsError::new(ErrorKind::DivideByZero))
        } else {
            Ok(amount(self.value_ref().div(rhs), self.currency()))
        }
    }
}
impl<'a> core::ops::Div<& 'a BigDecimal> for &Amount {
    type Output = Result<Amount, AmountOpsError>;
    fn div(self, rhs: & 'a BigDecimal) -> Self::Output {
        if rhs.is_zero() {
            Err(AmountOpsError::new(ErrorKind::DivideByZero))
        } else {
            Ok(amount((&self.value_ref()).div(rhs), self.currency()))
        }
    }
}



#[derive(Debug, thiserror::Error)]
#[derive(Copy, Clone)]
#[derive(PartialEq)]
pub enum ErrorKind {
    #[error("Different currencies ({0},{1})")]
    DifferentCurrencies(Currency,Currency),
    #[error("Divide by zero")]
    DivideByZero,
    #[error("Incorrect")]
    Incorrect,
}

#[derive(thiserror::Error)]
#[derive(mvv_static_error_macro::MyStaticStructError)]
pub struct AmountOpsError {
    pub kind: ErrorKind,
    pub backtrace: BacktraceCell,
    // #[source]
    pub source: ErrorSource,
}

#[derive(mvv_static_error_macro::MyStaticStructErrorSource)]
#[struct_error_type(AmountOpsError)]
pub enum ErrorSource {
    // #[error("No source")]
    NoSource,
    // Actually there should be BigDecimal errors...
    // but it !panics! in case of dividing by zero,
    // and seems to do not use other errors ?!
}
