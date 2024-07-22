

fn remove_space_chars_impl(str: &str) -> String {
    let mut res = String::with_capacity(str.len());
    let mut is_prev_non_space_char_alpha_or_digit = false;
    let mut is_prev_space = false;
    str.chars().for_each(|ch|{

        if ch.is_ascii_whitespace() {
            // we will add ' ' or not add later depending on prev and next non-space chars
        }
        else if ch.is_ascii_alphanumeric() {
            if is_prev_space && is_prev_non_space_char_alpha_or_digit {
                res.push(' ');
            }
            res.push(ch);
            is_prev_non_space_char_alpha_or_digit = true;
        }
        else {
            res.push(ch);

            is_prev_non_space_char_alpha_or_digit = false;
        }

        is_prev_space = ch.is_ascii_whitespace();
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
