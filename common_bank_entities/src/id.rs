use core::str::FromStr;
use mvv_common::unchecked::UncheckedResultUnwrap;
// -------------------------------------------------------------------------------------------------


// Using separate manual var makes sense if we plan to reuse it (in several validators).
//
lazy_static::lazy_static! {
    static ref ID_REGEX: regex::Regex = regex::Regex::new(r#"^[0-9A-Za-z_\-]+$"#).unchecked_unwrap();
}

#[nutype::nutype(
    // requires 1.80 or later
    // validate(not_empty, len_char_min = 1, len_char_max = 320, regex = r#"^[0-9A-Za-z_\-]+$"#),
    validate(not_empty, len_char_min = 1, len_char_max = 320, regex = ID_REGEX),
    derive(Debug, Display, PartialEq, Clone, Serialize, Deserialize),
)]
pub struct Id (
    // Not 'validator' nor 'validify' supports new-type.
    // #[validate(length(min=1, max=320, regex = "[^0-9a-zA-Z_-]"))]
    // For that reason 'nutype' is used.
    String);



#[inherent::inherent]
impl FromStr for Id {
    type Err = parse::IdFormatError;
    pub fn from_str(str: &str) -> Result<Self, parse::IdFormatError> {
        Ok(Id::try_new(str.to_string()) ?)
    }
}



pub mod parse {
    use crate::id::IdError;
    use mvv_common::backtrace::BacktraceCell;
    use crate::error::DataFormatError;

    // Probably we can remove them and use only generated by 'nutype' IdError...
    // However, it has a bit strange Display impl :-(
    //
    #[derive(Debug, thiserror::Error)]
    #[derive(Copy, Clone)]
    pub enum ErrorKind {
        #[error("Incorrect ID format")]
        IncorrectIdFormat
    }

    #[derive(thiserror::Error)]
    #[derive(mvv_error_macro::StructError)]
    pub struct IdFormatError {
        pub kind: ErrorKind,
        pub backtrace: BacktraceCell,
    }

    impl DataFormatError for IdFormatError { }

    impl From<IdError> for IdFormatError {
        fn from(_value: IdError) -> Self {
            IdFormatError::new(ErrorKind::IncorrectIdFormat)
        }
    }
}


#[cfg(test)]
mod tests {
    use regex::Regex;
    use mvv_common::unchecked::UncheckedResultUnwrap;
    use mvv_common::test::{TestResultUnwrap, TestDisplayStringOps, TestDebugStringOps };
    use super::*;

    lazy_static::lazy_static! {
        static ref ID_REGEX: Regex = Regex::new(r#"^[0-9A-Za-z_\-]+$"#).unchecked_unwrap();
    }

    #[test]
    fn validate_id_regex() {
        let is_matched = ID_REGEX.is_match("ЗапорізькийКозак");
        assert_eq!(is_matched, false);

        let is_matched = ID_REGEX.is_match("abcdef  non-ascii-alphas : forbidden ЗапорізькийКозак");
        assert_eq!(is_matched, false);

        let is_matched = ID_REGEX.is_match("abcdef forbidden ЗапорізькийКозак abcdef");
        assert_eq!(is_matched, false);

        let is_matched = ID_REGEX.is_match(" : $% forbidden ЗапорізькийКозак abcdef");
        assert_eq!(is_matched, false);

        let is_matched = ID_REGEX.is_match("abcdef-_");
        assert_eq!(is_matched, true);
    }

    #[test]
    fn validate_id_by_new() {
        assert_eq!(
            Id::try_new("abcdef  non-ascii-alphas : forbidden ЗапорізькийКозак".to_owned()),
            Err(IdError::RegexViolated),
        )
    }

    #[test]
    #[should_panic(expected = "IdFormatError { kind: IncorrectIdFormat")]
    fn validate_id_by_from_str() {
        let _id = Id::from_str("abcdef  non-ascii-alphas : forbidden ЗапорізькийКозак").test_unwrap();
        assert!(false);
    }

    #[test]
    fn validate_id_2() {
        let id = Id::try_new("abcd-12_34".to_owned()).test_unwrap();
        assert_eq!(id.to_test_string(), "abcd-12_34");
        assert_eq!(id.to_test_debug_string(), r#"Id("abcd-12_34")"#);
    }

    #[test]
    fn serialize_id() {
        let id = Id::try_new("abcd-12_34".to_owned()).test_unwrap();

        let json_str = serde_json::to_string(&id).test_unwrap();
        assert_eq!(json_str, r#""abcd-12_34""#);
    }

    #[test]
    fn deserialize_id() {
        let id: Id = serde_json::from_str(r#""abcd-12_34""#).test_unwrap();
        assert_eq!(id, Id::try_new("abcd-12_34").test_unwrap());
        assert_eq!(id.to_test_string(), "abcd-12_34");
    }

    #[test]
    #[should_panic(expected = "Error(\"Id violated the regular expression. Expected valid Id\"")]
    fn deserialize_wrong_id() {
        let _id: Id = serde_json::from_str(r#""abcd-12_34 ЗапорізькийКозак""#).test_unwrap();
    }
}