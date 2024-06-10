use bigdecimal::BigDecimal;
use crate::entities::currency::Currency;
// use crate::entities::currency::Currency;       // ++
// use ::project01::entities::currency::Currency; // --
// use project01::entities::currency::Currency;   // --
// use self::super::currency::Currency;           // ++
// use super::currency::Currency;                 // ++


// #[derive(Debug)]
#[derive(PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)] // TODO: move it to DTO
#[readonly::make]
pub struct Amount {
    #[serde(with = "crate::entities::serde_json_bd::bd_with")]
    pub value: BigDecimal,
    pub currency: Currency,
}

pub struct AmountParts {
    pub value: BigDecimal,
    pub currency: Currency,
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
/*
impl serde::Serialize for Amount {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        // serialize_amount_as_string(self, serializer)
        serialize_amount_as_struct(self, serializer)
    }
}
impl<'de> serde::Deserialize<'de> for Amount {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        // deserialize_amount_as_string(deserializer)
        deserialize_amount_as_struct(deserializer)
    }
}


fn serialize_amount_as_struct<S>(amount: &Amount, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
    use serde::ser::SerializeStruct;
    use crate::entities::serde_json_bd::BDRefSerdeWrapper;

    // let bd_wrapper = BDRefSerdeWrapper(&amount.value);
    // let currency = amount.currency.to_string();
    //
    // let mut s = serializer.serialize_struct("amount", 2) ?;
    // s.serialize_field("value", &bd_wrapper) ?;  // T O D O: use Display
    // s.serialize_field("currency", &currency) ?; // T O D O: use &str
    // s.end()

    // let bd_wrapper = BDRefSerdeWrapper(&amount.value);
    // let currency = amount.currency.to_string();

    let mut s = serializer.serialize_struct("amount", 2) ?;
    s.serialize_field("value", &BDRefSerdeWrapper(&amount.value)) ?;
    s.serialize_field("currency", &amount.currency) ?;
    s.end()
}

// TODO: add test
#[allow(dead_code)]
fn serialize_amount_as_string<S>(amount: &Amount, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
    use crate::util::serde_json::serialize_as_display_string;
    serialize_as_display_string(serializer, &amount)
}

// TODO: add test
#[allow(dead_code)]
fn deserialize_amount_as_string<'de, D>(deserializer: D) -> Result<Amount, D::Error>
    where D: serde::Deserializer<'de> {

    use crate::util::serde_json::deserialize_as_from_str;
    deserialize_as_from_str(deserializer)
}

fn deserialize_amount_as_struct<'de, D>(deserializer: D) -> Result<Amount, D::Error>
    where D: serde::Deserializer<'de> {

    use crate::entities::serde_json_bd::BDSerdeWrapper;
    use crate::entities::currency::CurrencyFormatError;
    use bigdecimal::ParseBigDecimalError;
    use serde::de::{ Visitor, MapAccess, Error };

    struct FieldVisitor;
    impl<'de> Visitor<'de> for FieldVisitor {
        type Value = Amount;

        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
            // actually it should not be used in our case
            write!(formatter, r#"{{ value: 1234.5678, currency: "EUR" }}"#)
        }
        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error> where A: MapAccess<'de> {

            use parse::{ ParseAmountError, ErrorKind };

            let mut unexpected_count = 0;
            let mut amount_value: Option<Result<BigDecimal, ParseBigDecimalError>> = None;
            let mut amount_currency: Option<Result<Currency, CurrencyFormatError>> = None;

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
                                if let Ok::<BDSerdeWrapper,_>(v) = map.next_value() {
                                    amount_value = Some(Ok(v.0))
                                }
                            }
                            _ => { unexpected_count += 1 }
                        }
                    }
                }
            };

            let amount_value: Result<BigDecimal, ParseBigDecimalError> = amount_value
                .ok_or_else(|| ParseAmountError::new(ErrorKind::NoAmount))
                .map_err(Error::custom) ?;

            let amount_currency = amount_currency
                .ok_or_else(|| ParseAmountError::new(ErrorKind::NoCurrency))
                .map_err(Error::custom) ?;

            let amount_value    = amount_value
                .map_err(|amount_err| ParseAmountError::with_from(ErrorKind::IncorrectAmount, amount_err))
                .map_err(Error::custom) ?;

            let amount_currency = amount_currency
                .map_err(|cur_err| ParseAmountError::with_from(ErrorKind::IncorrectCurrency, cur_err))
                .map_err(Error::custom) ?;

            if unexpected_count != 0 {
                // Seems it never works because list of expected fields is specified in call of deserialize_struct().
                return Err(Error::custom("Amount json block has unexpected items."));
            }

            Ok(Amount { value: amount_value, currency: amount_currency })
        }
    }
    let v = FieldVisitor{};
    deserializer.deserialize_struct("amount", &["value", "currency",], v)
}
*/

impl core::fmt::Debug for Amount {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Amount {{ {} {} ({:?}) }}", self.value, self.currency, self.value)
    }
}

pub mod parse {
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
impl crate::util::string::DisplayValueExample for Amount {
    fn display_value_example() -> &'static str { r#""1234.5678 EUR""# }
}


impl Amount {

    pub fn with_str_amount(amount: &str, currency: Currency) -> Result<Self, parse::ParseAmountError> {
        use core::str::FromStr;

        let bd: Result<BigDecimal, parse::ParseAmountError> = BigDecimal::from_str(amount)
            .map_err(|bd_err|parse::ParseAmountError::with_from(
                parse::ErrorKind::IncorrectAmount, bd_err));
        return bd.map(|am| Amount { value: am, currency } );
    }

    pub fn with_str_amount_unchecked(amount: &str, currency: Currency) -> Self {
        use crate::util::UncheckedResultUnwrap;
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

    pub fn value_bd_ref(&self) -> bigdecimal::BigDecimalRef<'_> {
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

    pub fn move_out(self) -> AmountParts {
        AmountParts { value: self.value, currency: self.currency }
    }
}

// Just short alias (similar to kotlin style)
#[inline]
pub fn amount(amount: BigDecimal, currency: Currency) -> Amount { Amount::new(amount, currency) }



#[inherent::inherent]
impl core::str::FromStr for Amount {
    type Err = parse::ParseAmountError;

    #[inline]
    pub fn from_str(s: &str) -> Result<Amount, parse::ParseAmountError> { crate::entities::parse_amount::parse_amount(s) }
}
