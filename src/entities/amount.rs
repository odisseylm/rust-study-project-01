use std::str::{ FromStr };
use bigdecimal::{ BigDecimal, BigDecimalRef, ParseBigDecimalError };
use crate::entities::currency::{ Currency, CurrencyFormatError };
use serde::{ Deserialize, Deserializer, Serialize, Serializer };
use serde::de::{ EnumAccess, Error, MapAccess, SeqAccess, Visitor };
use serde::ser::SerializeStruct;
// use crate::entities::currency::Currency;       // ++
// use ::project01::entities::currency::Currency; // --
// use project01::entities::currency::Currency;   // --
// use self::super::currency::Currency;           // ++
// use super::currency::Currency;                 // ++
use crate::util::UncheckedResultUnwrap;


// #[derive(Debug)]
#[derive(PartialEq, Eq)]
// #[derive(Serialize, Deserialize)]
pub struct Amount {
    value: BigDecimal,
    currency: Currency,
}

/*
struct Person {
    name: String,
    age: u8,
    phones: Vec<String>,
}

// This is what #[derive(Serialize)] would generate.
impl Serialize for Person {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut s = serializer. serialize_struct("Person", 3)?;
        s. serialize_field("name", &self.name)?;
        s. serialize_field("age", &self.age)?;
        s. serialize_field("phones", &self.phones)?;
        s. end()
    }
}
*/

impl Serialize for Amount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let mut s = serializer.serialize_struct("amount", 2) ?;
            s.serialize_field("value", &self.value.to_string()) ?;  // TODO: use Display
            s.serialize_field("currency", &self.currency.to_string()) ?; // TODO: use &str
        s.end()
    }
}


// struct SSDD {}

// impl<'de> From<ParseAmountError> for MapAccess<'de>::Error {
// impl<'de> From<ParseAmountError> for serde::de::Error {
//     fn from(value: ParseAmountError) -> Self {
//         todo!()
//     }
// }

// impl<'de> MapAccess<'de>::Error for ParseAmountError {
// impl<'de> serde::de::Error for ParseAmountError {
// impl serde::de::Error for ParseAmountError {
//     fn custom<T>(msg: T) -> Self where T: Display {
//         todo!()
//     }
// }


// fn to_de_ser_err(err: ParseAmountError) -> Box<dyn serde::de::Error> {
// // fn to_de_ser_err<'de>(err: ParseAmountError) -> Box<dyn MapAccess<'de>::Error> {
//     todo!()
// }

fn to_de_ser_err_3232<'de, MA: MapAccess<'de>>(err: parse_amount::ParseAmountError) -> <MA as MapAccess<'de>>::Error {
    // let err: i32 = MA::Error::custom(err);
    // let err: <MA as MapAccess<'de>>::Error = <MA as MapAccess<'de>>::Error::custom(err);
    let err: <MA as MapAccess<'de>>::Error = MA::Error::custom(err);
    err
}
// fn to_de_ser_err_3233<MA: MapAccess>(err: ParseAmountError) -> <MA as MapAccess>::Error {
//     let err: <MA as MapAccess>::Error = MA::Error::custom(err);
//     err
// }


impl<'de> Deserialize<'de> for Amount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {

        struct FieldVisitor;
        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = Amount;

            fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
                // actually it should not be used in our case
                write!(formatter, r#"{{ value: 1234.5678, currency: EUR }}"#)
            }
            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {

                use parse_amount::{ ParseAmountError, ErrorKind };

                let mut unexpected_count = 0;
                let mut amount_value: Option<Result<BigDecimal, ParseBigDecimalError>> = None;
                let mut amount_currency: Option<Result<Currency, CurrencyFormatError>> = None;

                while let Ok::<Option<(&str,&str)>,_>(ref v) = map.next_entry() {
                    match v {
                        Some::<(&str,&str)>(ref v) => {
                            match v.0 {
                                "value"    => { amount_value    = Some(BigDecimal::from_str(v.1)) }
                                "currency" => { amount_currency = Some(Currency::from_str(v.1))   }
                                _ => { unexpected_count += 1 }
                            }
                        }
                        None => { break; }
                    }
                }

                let amount_value: Result<BigDecimal, ParseBigDecimalError> = amount_value
                    .ok_or_else(|| ParseAmountError::new(ErrorKind::NoAmount))
                    .map_err(|e|to_de_ser_err_3232::<'de, A>(e)) ?;

                let amount_currency = amount_currency
                    .ok_or_else(|| ParseAmountError::new(ErrorKind::NoCurrency))
                    .map_err(|e|to_de_ser_err_3232::<'de, A>(e)) ?;

                let amount_value    = amount_value
                    .map_err(|amount_err| ParseAmountError::with_from(ErrorKind::IncorrectAmount, amount_err))
                    .map_err(|e|to_de_ser_err_3232::<'de, A>(e)) ?;

                let amount_currency = amount_currency
                    .map_err(|cur_err| ParseAmountError::with_from(ErrorKind::IncorrectCurrency, cur_err))
                    .map_err(|e|to_de_ser_err_3232::<'de, A>(e)) ?;

                if unexpected_count != 0 {
                    // T O D O: hm... It never works because list of expected fields is specified in call deserialize_struct
                    return Err(A::Error::custom("Amount json block has unexpected items."));
                }

                Ok(Amount { value: amount_value, currency: amount_currency })
            }
        }
        let v = FieldVisitor{};
        deserializer.deserialize_struct("amount", &["value", "currency",], v)
    }

    fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error> where D: Deserializer<'de> {
        todo!()
    }
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
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
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



#[inherent::inherent]
impl FromStr for Amount {
    type Err = parse_amount::ParseAmountError;

    #[inline]
    pub fn from_str(s: &str) -> Result<Amount, parse_amount::ParseAmountError> { crate::entities::parse_amount::parse_amount(s) }
}
