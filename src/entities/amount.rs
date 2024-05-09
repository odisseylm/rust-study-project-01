use std::fmt::{ Display, Formatter };
use bigdecimal::{ BigDecimal, BigDecimalRef, ParseBigDecimalError };
use crate::entities::{ Currency };
use std::str::{ CharIndices, FromStr };


#[derive(Debug)]
pub struct Amount {
    value: BigDecimal,
    currency: Currency,
}

impl Display for Amount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.value, self.currency)
    }
}


impl Amount {

    pub fn with_str_amount(amount: &str, currency: Currency) -> Result<Self, ParseBigDecimalError> {
        let bd: Result<BigDecimal, ParseBigDecimalError> = BigDecimal::from_str(amount);
        return bd.map(|am| Amount { value: am, currency } );
    }

    pub fn with_str_amount_unchecked(amount: &str, currency: Currency) -> Self {
        let bd: Result<BigDecimal, ParseBigDecimalError> = BigDecimal::from_str(amount);
        return bd.map(|am| Amount { value: am, currency } ).unwrap();
    }

    #[inline]
    pub fn new(amount: BigDecimal, currency: Currency) -> Amount {
        Amount { value: amount, currency }
    }

    // fn from_string(amount_with_currency: &str)

    pub fn currency(&self) -> Currency {
        self.currency.clone()
    }

    pub fn currency_ref(&self) -> &Currency {
        &self.currency
    }

    pub fn value(&self) -> &BigDecimal {
        &self.value
    }

    pub fn value_ref(&self) -> BigDecimalRef<'_> {
        self.value.to_ref()
    }

    pub fn with_value(&self, amount: BigDecimal) -> Amount {
        Amount { value: amount, currency: self.currency }
    }

    // I do not see sense to have function to 'change' currency (there are no such user/bank cases).
    // pub fn with_currency(&self, currency: Currency) -> Amount {
    //     Amount { value: self.value.clone(), currency }
    // }
}

#[inline]
pub fn amount(amount: BigDecimal, currency: Currency) -> Amount { Amount::new(amount, currency) }


fn parse_amount(s: &str) -> Result<Amount, ParseAmountError> {
    let s = s.trim();
    let chars_ind: CharIndices = s.char_indices();

    let mut last_space_bytes_offset: usize = usize::MAX;

    chars_ind.for_each(|it|{
        let byte_index: usize = it.0;
        let ch: char = it.1;

        if ch.is_ascii_whitespace() { last_space_bytes_offset = byte_index }
    });

    if last_space_bytes_offset == usize::MAX {
        return Err(ParseAmountError::NoCurrencyError)
    }

    let amount_and_cur = s.split_at(last_space_bytes_offset);
    let str_amount = amount_and_cur.0.trim_end();
    let str_cur = amount_and_cur.1.trim_start();

    let cur_res = Currency::from_str(str_cur);
    if cur_res.is_err() { return Err(ParseAmountError::ParseCurrencyError) }

    let amount_res = BigDecimal::from_str(str_amount);
    if cur_res.is_err() { return Err(ParseAmountError::ParseAmountError(amount_res.err().unwrap())); } // TODO: rewrite

    // return Err(ParseAmountError::ParseCurrencyError);
    return Ok(Amount::new(amount_res.unwrap(), cur_res.unwrap()));
}



#[derive(Debug, PartialEq, Clone)]
pub enum ParseAmountError {
    NoCurrencyError,
    ParseCurrencyError,
    ParseAmountError(ParseBigDecimalError),
}

impl FromStr for Amount {
    type Err = ParseAmountError;

    #[inline]
    fn from_str(s: &str) -> Result<Amount, Self::Err> {
        parse_amount(s)
    }
}
