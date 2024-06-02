use std::str::{ FromStr };
use bigdecimal::{ BigDecimal, BigDecimalRef, ParseBigDecimalError };
use crate::entities::currency::{ Currency, CurrencyFormatError };
use serde::{ Deserialize, Deserializer, Serialize, Serializer };
use serde::de::{EnumAccess, Error, MapAccess, SeqAccess, Visitor};
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

                /*
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
                */

                while let Ok::<Option<&str>,_>(key) = map.next_key() {
                    match key {
                        None => { break }
                        Some(key) => {
                            match key {
                                "currency" => {
                                    if let Ok::<&'de str,_>(v) = map.next_value() {
                                        amount_currency = Some(Currency::from_str(v))
                                    }
                                }
                                "value" => {
                                    println!("### some amount value...");

                                    // serde::de::value::Int64(map.next_value()?)
                                    // serde::de::value::

                                    if let Ok::<RawValueWrapper,_>(v) = map.next_value() {
                                        println!("### bd: {:?}", v);
                                        let raw_v: &serde_json::value::RawValue = v.0;
                                        let as_str: &str = raw_v.get();
                                        let as_str: &str = as_str.trim();
                                        let as_str: &str = as_str.strip_prefix("\"").unwrap_or(as_str);
                                        let as_str: &str = as_str.strip_suffix("\"").unwrap_or(as_str);
                                        let as_str: &str = as_str.trim();

                                        amount_value = Some(BigDecimal::from_str(as_str))
                                    }
                                    // if let Ok::<BDSerdeWrapper,_>(v) = map.next_value() {
                                    //     println!("### bd: {}", v);
                                    //     amount_value = Some(BigDecimal::from_str("888888"))
                                    // }

                                    // if let Ok::<BigDecimal,_>(v) = map.next_value() {
                                    //     let sss = v.to_string();
                                    //     println!("### bd: {}", v);
                                    //     amount_value = Some(BigDecimal::from_str("888888"))
                                    // }

                                    if let Ok::<serde_json::Value,_>(v) = map.next_value() {
                                        amount_value = Some(BigDecimal::from_str("888888"))
                                    }

                                    if let Ok::<&str,_>(v) = map.next_value() {
                                        amount_value = Some(BigDecimal::from_str(v))
                                    }
                                    if let Ok::<serde_json::Value,_>(v) = map.next_value() {
                                        amount_value = Some(BigDecimal::from_str("77777"))
                                    }
                                    if let Ok::<serde_json::Number,_>(v) = map.next_value() {
                                        amount_value = Some(BigDecimal::from_str("77777"))
                                    }
                                    // if let Ok::<serde_json::de::StrRead,_>(v) = map.next_value() {
                                    //     amount_value = Some(BigDecimal::from_str("77777"))
                                    // }
                                    // if let Ok::<serde_json::de::SliceRead,_>(v) = map.next_value() {
                                    //     amount_value = Some(BigDecimal::from_str("77777"))
                                    // }
                                    if let Ok::<&[u8],_>(v) = map.next_value() {
                                        amount_value = Some(BigDecimal::from_str("77777"))
                                    }
                                    // if let Ok::<serde_yaml::Number,_>(v) = map.next_value() {
                                    //     amount_value = Some(BigDecimal::from_str("77777"))
                                    // }
                                    if let Ok::<f64,_>(v) = map.next_value() {
                                        amount_value = Some(BigDecimal::from_str(v.to_string().as_str()))
                                    }
                                    if let Ok::<f32,_>(v) = map.next_value() {
                                        amount_value = Some(BigDecimal::from_str(v.to_string().as_str()))
                                    }

                                    if let Ok::<i8,_>(v) = map.next_value() {
                                        amount_value = Some(BigDecimal::from_str("888888"))
                                    }
                                    if let Ok::<i16,_>(v) = map.next_value() {
                                        amount_value = Some(BigDecimal::from_str("888888"))
                                    }
                                    if let Ok::<i32,_>(v) = map.next_value() {
                                        amount_value = Some(BigDecimal::from_str("888888"))
                                    }
                                    if let Ok::<i64,_>(v) = map.next_value() {
                                        amount_value = Some(BigDecimal::from_str("888888"))
                                    }
                                    if let Ok::<i128,_>(v) = map.next_value() {
                                        amount_value = Some(BigDecimal::from_str("888888"))
                                    }

                                    if let Ok::<u8,_>(v) = map.next_value() {
                                        amount_value = Some(BigDecimal::from_str("888888"))
                                    }
                                    if let Ok::<u16,_>(v) = map.next_value() {
                                        amount_value = Some(BigDecimal::from_str("888888"))
                                    }
                                    if let Ok::<u32,_>(v) = map.next_value() {
                                        amount_value = Some(BigDecimal::from_str("888888"))
                                    }
                                    if let Ok::<u64,_>(v) = map.next_value() {
                                        amount_value = Some(BigDecimal::from_str("888888"))
                                    }
                                    if let Ok::<u128,_>(v) = map.next_value() {
                                        amount_value = Some(BigDecimal::from_str("888888"))
                                    }
                                }
                                _ => { unexpected_count += 1 }
                            }
                        }
                    }
                };


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

    fn deserialize_in_place<D>(_deserializer: D, _place: &mut Self) -> Result<(), D::Error> where D: Deserializer<'de> {
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


#[derive(Deserialize, Debug)]
struct RawValueWrapper<'a>(
    #[serde(borrow)]
    // &'a serde_json::raw::RawValue
    &'a serde_json::value::RawValue
);


#[derive(Debug)]
pub struct BDSerdeWrapper(BigDecimal);

impl core::fmt::Display for BDSerdeWrapper {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct BDRefSerdeWrapper<'a>(& 'a BigDecimal);

impl<'se> Serialize for BDRefSerdeWrapper<'se> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        let as_string = self.0.to_string();
        let as_bytes = as_string.as_bytes();
        serializer.serialize_bytes(as_bytes)
    }
}

impl<'de> Deserialize<'de> for BDSerdeWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
        struct FV;
        impl<'de> Visitor<'de> for FV {
            type Value = BDSerdeWrapper;
            fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
                // todo!()
                write!(f, "44444")
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E> where E: Error {
                todo!()
            }

            fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_char<E>(self, v: char) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_none<E>(self) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: Deserializer<'de> {
                println!("fuck");
                todo!()
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E> where E: Error {
                println!("fuck");
                todo!()
            }

            fn visit_newtype_struct<D>(self, deserializer: D) -> Result<Self::Value, D::Error> where D: Deserializer<'de> {
                println!("fuck");
                deserializer.deserialize_any(FV{});
                todo!()
            }

            fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
                println!("fuck");
                todo!()
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {
                println!("fuck");
                todo!()
            }

            fn visit_enum<A>(self, data: A) -> Result<Self::Value, A::Error> where A: EnumAccess<'de> {
                println!("fuck");
                todo!()
            }

            fn __private_visit_untagged_option<D>(self, _: D) -> Result<Self::Value, ()> where D: Deserializer<'de> {
                println!("fuck");
                todo!()
            }
        }

        let v = FV;
        // Does NOT work!
        // deserializer.deserialize_bytes(v)
        // deserializer.deserialize_byte_buf(v)

        // works but converts value to f64 with decreasing precision
        // deserializer.deserialize_any(v)

        // deserializer.deserialize_ignored_any(v)
        //deserializer.deserialize_str(v);
        deserializer.deserialize_newtype_struct("fuck890", v)
    }

    // fn deserialize_in_place<D>(deserializer: D, place: &mut Self) -> Result<(), D::Error> where D: Deserializer<'de> {
    //     println!("fuck");
    //     todo!()
    // }
}



#[inherent::inherent]
impl FromStr for Amount {
    type Err = parse_amount::ParseAmountError;

    #[inline]
    pub fn from_str(s: &str) -> Result<Amount, parse_amount::ParseAmountError> { crate::entities::parse_amount::parse_amount(s) }
}
