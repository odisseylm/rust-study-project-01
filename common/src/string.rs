// use const_format::ascii_str;

use core::fmt::{ self, Debug };

#[derive(Debug, Clone)]
pub enum StaticRefOrString {
    Ref(&'static str),
    String(String),
    // We can put there other type of strings
}

impl StaticRefOrString {
    fn as_str(&self) -> &str {
        match self {
            StaticRefOrString::Ref(ref str) => str,
            StaticRefOrString::String(ref str) => str.as_str(),
        }
    }
}
impl fmt::Display for StaticRefOrString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}
pub trait DisplayValueExample : core::fmt::Display {
    fn display_value_example() -> &'static str;
}


pub fn substring_count(str: &str, sub_str: &str) -> usize {
    let aa: core::str::Matches<&str> = str.matches(sub_str);
    aa.count()
}

/*
pub fn ascii_substring_count(str: &str, sub_str: &[u8]) -> usize {

    let mut sub_string = String::with_capacity(sub_str.len());
    sub_str.iter().for_each(|b| {sub_string.push(*b as char)});
    let sub_str = sub_string.as_str();

    let mut count: usize = 0;
    let mut found_byte_offset: Option<usize> = str.find(sub_str);

    while let Some(found_byte_offset_val) = found_byte_offset {
        count += 1;

        let next_start = found_byte_offset_val + sub_str.len();

        found_byte_offset =
            if next_start < str.len() { str.find(sub_str) }
            else { None };
    }

    count
}
*/

/*
#[extension_trait::extension_trait]
pub impl StringOps for String {
    // Error ?!
    fn remove_suffix(mut self, suffix: &str) -> Result<String, String> {
        if self.ends_with(suffix) {
            self.truncate(self.len() - suffix.len());
            Ok(self)
        } else {
            Err(self)
        }
    }
}
*/


pub fn remove_suffix(mut str: String, suffix: &str) -> Result<String, String> {
    if str.ends_with(suffix) {
        str.truncate(str.len() - suffix.len());
        Ok(str)
    } else {
        Err(str)
    }
}

pub fn remove_optional_suffix(mut str: String, suffix: &str) -> String {
    if str.ends_with(suffix) {
        str.truncate(str.len() - suffix.len());
    }
    str
}
