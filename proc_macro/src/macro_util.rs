

fn remove_space_chars_impl(str: &str) -> String {
    let mut res = String::with_capacity(str.len());
    str.chars().for_each(|ch|{
        if !ch.is_ascii_whitespace() {
            res.push(ch);
        }
    });
    res
}

#[allow(dead_code)]
pub trait StringOp {
    fn remove_space_chars(&self) -> String;
}
impl StringOp for String {
    fn remove_space_chars(&self) -> String { remove_space_chars_impl(self.as_str()) }
}
impl StringOp for &str {
    fn remove_space_chars(&self) -> String { remove_space_chars_impl(self) }
}


//--------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    fn remove_spaces_from_type_str(str_type: &str) -> String {
        super::remove_space_chars_impl(&str_type.to_owned())
    }

    #[test]
    fn test_remove_spaces_from_type() {
        assert_eq!(remove_spaces_from_type_str("anyhow :: Error"), "anyhow::Error");
        assert_eq!(remove_spaces_from_type_str("anyhow::Error"), "anyhow::Error");
        assert_eq!(remove_spaces_from_type_str(" anyhow :: Error"), "anyhow::Error");
        assert_eq!(remove_spaces_from_type_str("  anyhow  ::  Error  "), "anyhow::Error");

        assert_eq!(remove_spaces_from_type_str("Box<dyn std::error::Error>"), "Box<dyn std::error::Error>");
        assert_eq!(remove_spaces_from_type_str(" Box < dyn std :: error :: Error > "), "Box<dyn std::error::Error>");
        assert_eq!(remove_spaces_from_type_str("  Box  <  dyn  std  ::  error  ::  Error  >  "), "Box<dyn std::error::Error>");
    }
}
