use core::str::FromStr;
use regex::Regex;
// #[derive(Debug, Clone, PartialEq, derive_more::Display)]
// #[derive(serde::Serialize, serde::Deserialize)]
// #[display(fmt = "{}", "0")]
// #[nutype(
//     validate(not_empty, len_char_max = 20),
//     derive(Debug, PartialEq),
// )]

// Not 'validator' nor 'validify' supports new-type.
// #[derive(validator::Validate)]


lazy_static::lazy_static! {
    // Note: this regex is very simplified.
    // In reality you'd like to use a more sophisticated regex for email validation.
    // static ref EMAIL_REGEX: Regex = Regex::new("^\\w+@\\w+\\.\\w+$").unwrap();
    static ref ID_REGEX: Regex = Regex::new(r#"^[0-9A-Za-z_\-]+$"#).unwrap();
    // static ref ID_REGEX: Regex = Regex::new(r#"^[0-9A-Za-z]$"#).unwrap();
    // static ref ID_REGEX: Regex = Regex::new(r#"^[0-9]+$"#).unwrap();
}

// "[0-9a-zA-Z]" // [0-9A-Za-z_]
#[nutype::nutype(
    validate(not_empty, len_char_min = 1, len_char_max = 320, regex = ID_REGEX),
    derive(Debug, Display, PartialEq, Clone, Serialize, Deserialize),
)]
pub struct Id (
    // Not 'validator' nor 'validify' supports new-type.
    // #[validate(email)]
    // #[validate(length(min=1, max=320, regex = "[^0-9a-zA-Z_-]"))]
    String);

impl Id {
    pub fn move_out(self) -> String { self.into_inner() }
}


#[inherent::inherent]
impl FromStr for Id {
    type Err = parse::IdFormatError;
    pub fn from_str(str: &str) -> Result<Self, parse::IdFormatError> {
        Ok(Id::new(str.to_string()) ?)
    }
}



pub mod parse {
    use crate::entities::id::IdError;
    use crate::util::backtrace::BacktraceInfo;

    #[derive(Debug, thiserror::Error)]
    #[derive(Copy, Clone)]
    pub enum ErrorKind {
        #[error("Incorrect ID format")]
        IncorrectIdFormat
    }

    #[derive(thiserror::Error)]
    #[derive(static_error_macro::MyStaticStructError)]
    pub struct IdFormatError {
        pub kind: ErrorKind,
        pub backtrace: BacktraceInfo,
    }

    impl From<IdError> for IdFormatError {
        fn from(_value: IdError) -> Self {
            IdFormatError::new(ErrorKind::IncorrectIdFormat)
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::util::test_unwrap::TestSringOps;
    use crate::util::TestResultUnwrap;
    use super::*;

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
            Id::new("abcdef  non-ascii-alphas : forbidden ЗапорізькийКозак".to_owned()),
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
        let id = Id::new("abcd-12_34".to_owned()).test_unwrap();
        assert_eq!(id.to_test_string(), "abcd-12_34");
        assert_eq!(id.to_test_debug_string(), r#"Id("abcd-12_34")"#);
    }

    #[test]
    fn serialize_id() {
        let id = Id::new("abcd-12_34".to_owned()).test_unwrap();

        let json_str = serde_json::to_string(&id).test_unwrap();
        assert_eq!(json_str, r#""abcd-12_34""#);
    }

    #[test]
    fn deserialize_id() {
        let id: Id = serde_json::from_str(r#""abcd-12_34""#).test_unwrap();
        assert_eq!(id, Id::new("abcd-12_34").test_unwrap());
        assert_eq!(id.to_test_string(), "abcd-12_34");
    }

    #[test]
    #[should_panic(expected = "Error(\"Id violated the regular expression. Expected valid Id\"")]
    fn deserialize_wrong_id() {
        let id: Id = serde_json::from_str(r#""abcd-12_34 ЗапорізькийКозак""#).test_unwrap();
    }
}