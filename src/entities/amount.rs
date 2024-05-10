use std::fmt::{ Display, Formatter };
use bigdecimal::{ BigDecimal, BigDecimalRef, ParseBigDecimalError };
use crate::entities::{Currency, CurrencyFormatError};
use std::str::{ FromStr };


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
        Amount::with_str_amount(amount, currency).unwrap()
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
    let last_space_bytes_offset_opt: Option<usize> = s.rfind(|ch: char|{ ch.is_ascii_whitespace() });

    // IMPL with matches:
    //   ++ no unsafe 'unwrap'
    //   +  less-more short, but with IFs a bit shorter
    //
    match last_space_bytes_offset_opt {
        None => { Err(ParseAmountError::NoCurrencyError) }
        Some(last_space_bytes_offset) => {
            let (str_amount, str_cur) = s.split_at(last_space_bytes_offset);
            let currency_res = Currency::from_str(str_cur.trim_start());

            match currency_res {
                Err(cur_parse_err) => { Err(ParseAmountError::ParseCurrencyError(cur_parse_err)) }
                Ok(currency) => {
                    let amount_res = BigDecimal::from_str(str_amount.trim_end());

                    match amount_res {
                        Err(amount_parse_err) => { Err(ParseAmountError::ParseAmountError(amount_parse_err)) }
                        Ok(amount) => { Ok(Amount::new(amount, currency)) }
                    }
                }
            }
        }
    }

    /*
    // IMPL with functional approach:
    //   -- does NOT work now -> no flat_map
    //
    let res1: Result<usize, ParseAmountError> = last_space_bytes_offset.ok_or(ParseAmountError::NoCurrencyError);

    // there is no flat_map ??!!
    return res1.map(|last_space_bytes_offset: usize| {
        let amount_and_cur = s.split_at(last_space_bytes_offset);
        let str_amount = amount_and_cur.0.trim_end();
        let str_cur = amount_and_cur.1.trim_start();

        let cur_res = Currency::from_str(str_cur);
        if cur_res.is_err() {
            Err(ParseAmountError::ParseCurrencyError)
        } else {
            let amount_res = BigDecimal::from_str(str_amount);
            if cur_res.is_err() {
                Err(ParseAmountError::ParseAmountError(amount_res.err().unwrap()))
            } else {
                Ok(Amount::new(amount_res.unwrap(), cur_res.unwrap()))
            }
        }
    }).unwrap()
    */

    /*
    // IMPL with IFs and unwraps:
    //   ++ simple
    //   -- using unwrap()
    //
    if last_space_bytes_offset.is_none() {
        return Err(ParseAmountError::NoCurrencyError)
    }

    let amount_and_cur = s.split_at(last_space_bytes_offset.unwrap());
    let str_amount = amount_and_cur.0.trim_end();
    let str_cur = amount_and_cur.1.trim_start();

    let cur_res = Currency::from_str(str_cur);
    if cur_res.is_err() { return Err(ParseAmountError::ParseCurrencyError) }

    let amount_res = BigDecimal::from_str(str_amount);
    if cur_res.is_err() { return Err(ParseAmountError::ParseAmountError(amount_res.err().unwrap())); }

    // return Err(ParseAmountError::ParseCurrencyError);
    return Ok(Amount::new(amount_res.unwrap(), cur_res.unwrap()));
    */
}



#[derive(Debug, PartialEq, Clone)]
pub enum ParseAmountError {
    NoCurrencyError,
    ParseCurrencyError(CurrencyFormatError),
    ParseAmountError(ParseBigDecimalError),
}

impl FromStr for Amount {
    type Err = ParseAmountError;

    #[inline]
    fn from_str(s: &str) -> Result<Amount, Self::Err> { parse_amount(s) }
}
