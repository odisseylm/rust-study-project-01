use std::fmt::Write;
use std::ops::Deref;
use bigdecimal::BigDecimal;
use serde::Deserializer;
use bytes::BytesMut;


// -------------------------------------------------------------------------------------------------
//                                   BigDecimal wrappers
// -------------------------------------------------------------------------------------------------


#[derive(Debug)]
pub struct BDSerdeWrapper(pub BigDecimal);
#[derive(Debug)]
pub struct BDRefSerdeWrapper<'a>(pub & 'a BigDecimal);


impl core::fmt::Display for BDSerdeWrapper {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl<'a> core::fmt::Display for BDRefSerdeWrapper<'a> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}


impl Deref for BDSerdeWrapper {
    type Target = BigDecimal;
    fn deref(&self) -> &Self::Target { &self.0 }
}
impl<'a> Deref for BDRefSerdeWrapper<'a> {
    type Target = BigDecimal;
    fn deref(&self) -> &Self::Target { &self.0 }
}


impl<'se> serde::Serialize for BDRefSerdeWrapper<'se> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serialize_json_bd(self.0, serializer)
    }
}


impl<'de> serde::Deserialize<'de> for BDSerdeWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        deserialize_json_bd(deserializer).map(|bd| BDSerdeWrapper(bd))
    }
}
impl serde::Serialize for BDSerdeWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serialize_json_bd(&self.0, serializer)
    }
}


// -------------------------------------------------------------------------------------------------
//                           Serialize & Deserialize BigDecimal impl
// -------------------------------------------------------------------------------------------------

#[cfg(feature = "serde_json_raw_value")]
pub fn serialize_json_bd_as_raw_value<'se,S>(bd: &BigDecimal, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
    use serde::ser::Error;

    let as_string = bd.to_string();
    let raw_value = serde_json::value::RawValue::from_string(as_string).map_err(|err| Error::custom(err.to_string()) ) ?;

    serde::Serializer::serialize_newtype_struct(serializer, "BigDecimal", &raw_value)
}

pub fn serialize_json_bd_as_string<'se,S>(bd: &BigDecimal, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
    let as_string = bd.to_string();
    serde::Serializer::serialize_str(serializer, as_string.as_str())
}

pub fn serialize_json_bd_as_f64<'se,S>(bd: &BigDecimal, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
    use serde::ser::Error;
    use bigdecimal::ToPrimitive;

    let as_f64: f64 = bd.to_f64().ok_or_else(||Error::custom("Impossible to convert BigDecimal to f64.")) ?;
    serde::Serializer::serialize_f64(serializer, as_f64)
}

pub fn serialize_json_bd<'se,S>(bd: &BigDecimal, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
    #[cfg(feature = "serde_json_raw_value")]
    { serialize_json_bd_as_raw_value(bd, serializer) }

    #[cfg(not(feature = "serde_json_raw_value"))]
    { serialize_json_bd_as_f64(bd, serializer) }
}



#[cfg(feature = "serde_json_raw_value")]
pub fn deserialize_json_bd_as_raw_value<'de, D>(deserializer: D) -> Result<BigDecimal, D::Error> where D: serde::Deserializer<'de> {
    use serde_json::value::RawValue;
    let raw_val: &'de RawValue = <&'de RawValue as serde::Deserialize>::deserialize(deserializer) ?;

    let str: &str = raw_val.get();

    let str: &str = str.trim();
    let str: &str = str
        .strip_prefix('"').unwrap_or(str)
        .strip_suffix('"').unwrap_or(str)
        .trim();

    use std::str::FromStr;
    use serde::de::Error;
    let bd = BigDecimal::from_str(str).map_err(|err| Error::custom(err))?;
    return Ok::<BigDecimal, D::Error>(bd);
}


pub fn deserialize_json_bd_as_std_json_value<'de, D>(deserializer: D) -> Result<BigDecimal, D::Error> where D: serde::Deserializer<'de> {

    // if true { return todo!("Fuck 483483874678") }
    // const MAX_STR_LEN: usize = 64;
    // let mut buffer: [u8;MAX_STR_LEN] = [0;MAX_STR_LEN];

    use serde::de::{ Visitor, Error };
    // #[derive(Default)]
    struct FV;
    impl<'de> Visitor<'de> for FV {
        type Value = BigDecimal;
        fn expecting(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
            write!(f, "string or numeric format")
        }

        fn visit_i8<E>(self, v: i8) -> Result<Self::Value, E> where E: Error {
            Ok::<Self::Value, E>(BigDecimal::from(v))
        }
        fn visit_i16<E>(self, v: i16) -> Result<Self::Value, E> where E: Error {
            Ok::<Self::Value, E>(BigDecimal::from(v))
        }
        fn visit_i32<E>(self, v: i32) -> Result<Self::Value, E> where E: Error {
            Ok::<Self::Value, E>(BigDecimal::from(v))
        }
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E> where E: Error {
            Ok::<Self::Value, E>(BigDecimal::from(v))
        }
        fn visit_i128<E>(self, v: i128) -> Result<Self::Value, E> where E: Error {
            Ok::<Self::Value, E>(BigDecimal::from(v))
        }
        fn visit_u8<E>(self, v: u8) -> Result<Self::Value, E> where E: Error {
            Ok(BigDecimal::from(v))
        }
        fn visit_u16<E>(self, v: u16) -> Result<Self::Value, E> where E: Error {
            Ok(BigDecimal::from(v))
        }
        fn visit_u32<E>(self, v: u32) -> Result<Self::Value, E> where E: Error {
            Ok(BigDecimal::from(v))
        }
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E> where E: Error {
            Ok(BigDecimal::from(v))
        }
        fn visit_u128<E>(self, v: u128) -> Result<Self::Value, E> where E: Error {
            Ok(BigDecimal::from(v))
        }
        fn visit_f32<E>(self, v: f32) -> Result<Self::Value, E> where E: Error {
            // if true { return todo!("Fuck f32") }
            // very-very bad approach with loosing precision
            // use bigdecimal::FromPrimitive;
            // BigDecimal::from_f32(v).ok_or_else(||Error::custom("Wrong f32 big-decimal format"))

            // let as_string = v.to_string(); // TODO: remove this heap allocation
            // use std::str::FromStr;
            // BigDecimal::from_str(as_string.as_str()).map_err(|err|Error::custom(err))

            // let mut str_buf = String::new();
            // write!(str_buf, "{}", v).unwrap(); // TODO: fdfdf
            // use std::str::FromStr;
            // BigDecimal::from_str(str_buf.as_str()).map_err(|err|Error::custom(err))

            // let mut str_buf = String::new();
            // str_buf.write_fmt(format_args!("{0}", v)).unwrap();
            // use std::str::FromStr;
            // BigDecimal::from_str(str_buf.as_str()).map_err(|err| Error::custom(err))

            use std::str::{ self, FromStr };

            const MAX_STR_LEN: usize = 64;
            // let mut buffer: [u8;MAX_STR_LEN] = [0;MAX_STR_LEN];
            let mut buffer: [u8;MAX_STR_LEN] = [0;MAX_STR_LEN];
            let mut buffer = BytesMut::from(buffer.as_slice());
            buffer.write_fmt(format_args!("{0}", v)).map_err(|err| Error::custom(err)) ?;

            let as_str: &str = str::from_utf8(buffer.as_ref()).unwrap();
            BigDecimal::from_str(as_str).map_err(|err| Error::custom(err))

            // write!(&buffer, "{}", v).map_err(|err|Error::custom(err)) ?;
            // write!(&buffer, "{}", v).map_err(|err|Error::custom(err)) ?;

        }
        fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E> where E: Error {
            // very-very bad approach with loosing precision
            // use bigdecimal::FromPrimitive;
            // BigDecimal::from_f64(v).ok_or_else(||Error::custom("Wrong f64 big-decimal format"))

            // 13.346000000000000085265128291212022304534912109375

            // let as_string = v.to_string(); // TODO: remove this heap allocation
            // use std::str::FromStr;
            // BigDecimal::from_str(as_string.as_str()).map_err(|err|Error::custom(err))

            use std::str::{ self, FromStr };

            const MAX_STR_LEN: usize = 64;
            // let mut buffer: [u8;MAX_STR_LEN] = [0;MAX_STR_LEN];
            let mut buffer: [u8;MAX_STR_LEN] = [0;MAX_STR_LEN];
            let mut buffer = BytesMut::from(buffer.as_slice());
            buffer.write_fmt(format_args!("{0}", v)).map_err(|err| Error::custom(err)) ?;

            let as_str: &str = str::from_utf8(buffer.as_ref()).unwrap();
            BigDecimal::from_str(as_str).map_err(|err| Error::custom(err))
        }
        fn visit_char<E>(self, v: char) -> Result<Self::Value, E> where E: Error {
            let mut as_raw_str: [u8; 4] = [0; 4];
            let as_raw_str: &str = v.encode_utf8(&mut as_raw_str);
            use core::str::FromStr;
            BigDecimal::from_str(as_raw_str).map_err(|err|Error::custom(err))
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
            use core::str::FromStr;
            BigDecimal::from_str(v).map_err(|err|Error::custom(err))
        }
        fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E> where E: Error {
            use core::str::FromStr;
            BigDecimal::from_str(v).map_err(|err|Error::custom(err))
        }
        fn visit_string<E>(self, v: String) -> Result<Self::Value, E> where E: Error {
            use core::str::FromStr;
            BigDecimal::from_str(v.as_str()).map_err(|err|Error::custom(err))
        }
    }

    let v = FV;
    deserializer.deserialize_any(v)
}


pub fn deserialize_json_bd<'de, D>(deserializer: D) -> Result<BigDecimal, D::Error> where D: serde::Deserializer<'de> {

    #[cfg(feature = "serde_json_raw_value")]
    { deserialize_json_bd_as_raw_value(deserializer) }

    #[cfg(not(feature = "serde_json_raw_value"))]
    { deserialize_json_bd_as_std_json_value(deserializer) }
}


// -------------------------------------------------------------------------------------------------
//                           As serde BigDecimal serialize/deserialize module
// -------------------------------------------------------------------------------------------------

pub mod bd_with {
    use bigdecimal::BigDecimal;

    #[inline]
    pub fn serialize<'se,S>(bd: &BigDecimal, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        super::serialize_json_bd(bd, serializer)
    }

    #[inline]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<BigDecimal, D::Error> where D: serde::Deserializer<'de> {
        super::deserialize_json_bd::<'de, D>(deserializer)
    }
}

