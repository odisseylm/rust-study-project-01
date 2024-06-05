use crate::util::string::DisplayValueExample;

pub fn serialize_as_display_string<S,T>(serializer: S, value: &T) -> Result<S::Ok, S::Error>
    where S: serde::Serializer, T: core::fmt::Display {
    let value_as_str = value.to_string(); // TODO: try to avoid using heap allocation
    serializer.serialize_str(&value_as_str)
}


pub fn deserialize_as_from_str<'de,D,T>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: core::str::FromStr + DisplayValueExample,
        <T as core::str::FromStr>::Err: core::fmt::Display,{
    deserialize_as_from_str_with_expecting(deserializer, <T as DisplayValueExample>::display_value_example())
}

pub fn deserialize_as_from_str_with_expecting<'de,D,T>(deserializer: D, expecting_msg: &'static str) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: core::str::FromStr,
        <T as core::str::FromStr>::Err: core::fmt::Display,
{
    use core::str::FromStr;
    use serde::de::{ Error, Visitor };

    struct FieldVisitor<T> {
        expecting_msg: &'static str,
        _pd: core::marker::PhantomData<T>,
    }
    impl<'de, T: FromStr> Visitor<'de> for FieldVisitor<T>
        where T: FromStr, <T as FromStr>::Err: core::fmt::Display {

        type Value = T;

        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
            write!(formatter, "{}", self.expecting_msg)
        }
        fn visit_char<E>(self, v: char) -> Result<Self::Value, E> where E: Error {
            let mut as_raw_str: [u8; 4] = [0; 4];
            let as_raw_str: &str = v.encode_utf8(&mut as_raw_str);
            use core::str::FromStr;
            FromStr::from_str(as_raw_str).map_err(|err|Error::custom(err))
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> where E: Error {
            FromStr::from_str(v).map_err(Error::custom)
        }
        fn visit_borrowed_str<E>(self, v: &'de str) -> Result<Self::Value, E> where E: Error {
            FromStr::from_str(v).map_err(Error::custom)
        }
        fn visit_string<E>(self, v: String) -> Result<Self::Value, E> where E: Error {
            FromStr::from_str(v.as_str()).map_err(Error::custom)
        }
    }
    let v = FieldVisitor::<T>{ expecting_msg, _pd: core::marker::PhantomData };
    deserializer.deserialize_any(v)
}

// impl<'de> Deserialize<'de> for Amount {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {

// trait AsStringJsonSerde<'de,T,S,D> : serde::Serialize + serde::Deserialize<'de>
//     where
//         S: serde::Serializer,
//         T: core::fmt::Display,
//         Self: core::fmt::Display,
//         D: serde::Deserializer<'de>,
//         T: core::str::FromStr + DisplayValueExample,
//         <T as core::str::FromStr>::Err: core::fmt::Display,
//         Self: core::str::FromStr + DisplayValueExample,
//         <Self as core::str::FromStr>::Err: core::fmt::Display,
// {
//     fn serialize(&self, serializer: S) -> Result<S::Ok, S::Error> {
//         serialize_as_display_string(serializer, &self)
//     }
//     fn deserialize(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
//         deserialize_as_from_str(deserializer)
//     }
// }
// pub trait JsonSerdeAsStringSerializerAndDeserializer<'de,S,D> : serde::Serialize + serde::Deserialize<'de>
//     where
//         S: serde::Serializer,
//         D: serde::Deserializer<'de>,
//         Self: core::fmt::Display,
//         Self: core::str::FromStr + DisplayValueExample,
//         <Self as core::str::FromStr>::Err: core::fmt::Display,
// {
//     fn serialize(&self, serializer: S) -> Result<S::Ok, S::Error> {
//         serialize_as_display_string(serializer, &self)
//     }
//     fn deserialize(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
//         deserialize_as_from_str(deserializer)
//     }
// }
pub trait JsonSerdeAsStringSerializerImpl //: serde::ser::Serialize
    where Self: core::fmt::Display {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serialize_as_display_string(serializer, &self)
    }
}
pub trait JsonSerdeAsStringDeserializerImpl<'de,D> : serde::de::Deserialize<'de>
    where
        D: serde::Deserializer<'de>,
        Self: core::str::FromStr + DisplayValueExample,
        <Self as core::str::FromStr>::Err: core::fmt::Display,
{
    fn deserialize(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
        deserialize_as_from_str(deserializer)
    }
}

pub mod as_str {
    use crate::util::string::DisplayValueExample;

    pub fn serialize<S,T>(serializer: S, value: &T) -> Result<S::Ok, S::Error>
        where S: serde::Serializer, T: core::fmt::Display {
        super::serialize_as_display_string(serializer, value)
    }
    pub fn deserialize<'de,D,T>(deserializer: D) -> Result<T, D::Error>
        where
            D: serde::Deserializer<'de>,
            T: core::str::FromStr + DisplayValueExample,
            <T as core::str::FromStr>::Err: core::fmt::Display {
        super::deserialize_as_from_str(deserializer)
    }
}
