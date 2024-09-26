use crate::backtrace::BacktraceCell;
//--------------------------------------------------------------------------------------------------



#[derive(Debug, thiserror::Error)]
pub enum ConvertProtoEnumError {
    #[error("NoProtoEnumError")]
    NoProtoEnumError {
        proto_enum_index: i32,
        proto_enum_name: Option<&'static str>,
        model_enum_name: Option<&'static str>,
        // I do not think a backtrace is needed (now),
        // but let's remain such possibility for future compatibility
        backtrace: Option<BacktraceCell>,
    },
    #[error("NoRustEnumError")]
    NoModelEnumError {
        proto_enum_index: i32,
        proto_enum_name: Option<&'static str>,
        name: &'static str,
        // I do not think a backtrace is needed (now),
        // but let's remain such possibility for future compatibility
        backtrace: Option<BacktraceCell>,
    },
}
impl ConvertProtoEnumError {
    /// Use if unspecified is not allowed for your business logic (for mandatory field)
    pub fn unspecified_proto_enum_value(unspecified_index: i32) -> ConvertProtoEnumError {
        ConvertProtoEnumError::NoModelEnumError {
                proto_enum_index: unspecified_index,
                proto_enum_name: Some("Unspecified"),
                name: "",
                backtrace: None,
            }
    }
}

impl From<prost::UnknownEnumValue> for ConvertProtoEnumError {
    fn from(value: prost::UnknownEnumValue) -> Self {
        ConvertProtoEnumError::NoProtoEnumError {
            proto_enum_index: value.0,
            proto_enum_name: None,
            model_enum_name: None,
            backtrace: None,
        }
    }
}