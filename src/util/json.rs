// use serde::Deserialize;
// use serde_json::error::Result;
// use serde_with_macros::serde_as;

#[macro_export] macro_rules! json_str_ser_deser_impl {
    ($type_name:ty) => {

        impl serde::ser::Serialize for $type_name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
                crate::util::serde_json::serialize_as_display_string(serializer, &self)
            }
        }
        impl<'de> serde::Deserialize<'de> for $type_name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: serde::Deserializer<'de> {
                crate::util::serde_json::deserialize_as_from_str(deserializer)
            }
        }

    };
}
