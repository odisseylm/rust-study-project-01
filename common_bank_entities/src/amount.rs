use bigdecimal::BigDecimal;
use crate::currency::Currency;

pub mod parse;
pub mod ops;


#[derive(PartialEq, Eq)]
#[derive(educe::Educe)] #[educe(Debug)]
#[derive(derive_more::Display)]
#[display("{} {}", value, currency)]
#[readonly::make]
pub struct Amount {
    #[educe(Debug(method(crate::bd::bd_dbg_fmt)))]
    pub value: BigDecimal,
    pub currency: Currency,
}

impl mvv_common::string::DisplayValueExample for Amount {
    fn display_value_example() -> &'static str { r#""1234.5678 EUR""# }
}

pub struct AmountParts {
    pub value: BigDecimal,
    pub currency: Currency,
}


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
    use crate::serde_json_bd::BDRefSerdeWrapper;

    // let bd_wrapper = BDRefSerdeWrapper(&amount.value);
    // let currency = amount.currency.to_test_string();
    //
    // let mut s = serializer.serialize_struct("amount", 2) ?;
    // s.serialize_field("value", &bd_wrapper) ?;  // T O D O: use Display
    // s.serialize_field("currency", &currency) ?; // T O D O: use &str
    // s.end()

    // let bd_wrapper = BDRefSerdeWrapper(&amount.value);
    // let currency = amount.currency.to_test_string();

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

    use crate::serde_json_bd::BDSerdeWrapper;
    use crate::currency::CurrencyFormatError;
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

            use parse::{ AmountFormatError, ErrorKind };

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
                .ok_or_else(|| AmountFormatError::new(ErrorKind::NoAmount))
                .map_err(Error::custom) ?;

            let amount_currency = amount_currency
                .ok_or_else(|| AmountFormatError::new(ErrorKind::NoCurrency))
                .map_err(Error::custom) ?;

            let amount_value    = amount_value
                .map_err(|amount_err| AmountFormatError::with_from(ErrorKind::IncorrectAmount, amount_err))
                .map_err(Error::custom) ?;

            let amount_currency = amount_currency
                .map_err(|cur_err| AmountFormatError::with_from(ErrorKind::IncorrectCurrency, cur_err))
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


impl Amount {

    pub fn with_str_amount(amount: &str, currency: Currency) -> Result<Self, parse::AmountFormatError> {
        use core::str::FromStr;

        let bd: Result<BigDecimal, parse::AmountFormatError> = BigDecimal::from_str(amount)
            .map_err(|bd_err|parse::AmountFormatError::with_from(
                parse::ErrorKind::IncorrectAmount, bd_err));
        return bd.map(|am| Amount { value: am, currency } );
    }

    pub fn with_str_amount_unchecked(amount: &str, currency: Currency) -> Self {
        use mvv_common::unchecked::UncheckedResultUnwrap;
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
    //     Amount { value: self.value.c l o n e(), currency }
    // }

    pub fn into_parts(self) -> AmountParts {
        AmountParts { value: self.value, currency: self.currency }
    }
}

// Just short alias (similar to kotlin style)
#[inline]
pub fn amount(amount: BigDecimal, currency: Currency) -> Amount { Amount::new(amount, currency) }



#[inherent::inherent]
impl core::str::FromStr for Amount {
    type Err = parse::AmountFormatError;
    #[inline]
    pub fn from_str(s: &str) -> Result<Amount, parse::AmountFormatError> { parse::parse_amount(s) }
}
