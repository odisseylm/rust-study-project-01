use mvv_common::{
    generate_from_str_new_type_delegate, generate_pg_delegate_decode,
    generate_pg_delegate_encode, generate_pg_delegate_type_info,
    generate_simple_debug, generate_simple_display,
};
//--------------------------------------------------------------------------------------------------



#[derive(Clone, PartialEq, Eq)]
#[derive(serde::Serialize, serde::Deserialize)]
pub struct SecureString(String);

impl Drop for SecureString {
    fn drop(&mut self) {
        clear_string_chars(&mut self.0);
        // println!("### cleaning psw string");
    }
}

impl SecureString {
    #[inline]
    pub fn from_string(string: String) -> SecureString {
        SecureString(string)
    }
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

generate_simple_display! { SecureString, "Secure[...]" }
generate_simple_debug!   { SecureString, "Secure[...]" }

impl From<String> for SecureString {
    #[inline]
    fn from(string: String) -> Self {
        SecureString(string)
    }
}
impl From<&str> for SecureString {
    #[inline]
    fn from(str: &str) -> Self {
        SecureString(str.to_string())
    }
}
generate_from_str_new_type_delegate! { SecureString, String, core::convert::Infallible }
generate_pg_delegate_type_info!      { SecureString, String }
generate_pg_delegate_encode!         { SecureString, String }
generate_pg_delegate_decode!         { SecureString, String }


//---------------------- String cleaning impl ----------------------------
static ZERO_STR: &str = const_str::repeat!("\0", 1024);

pub fn clear_string_chars(str: &mut String) {
    clear_string_chars_impl(str, ZERO_STR)
}

fn clear_string_chars_impl(str: &mut String, zero_str: &str) {
    let len = str.len();
    if len == 0 {
        return;
    }

    // Good but with using unsafe
    /*
    let as_bytes = unsafe { str.as_bytes_mut() };
    for i in 0..as_bytes.len() {
        as_bytes[i] = 0;
    }
    */

    if len <= zero_str.len() {
        let zero_str_with_same_len: &str = &zero_str[0..len];
        str.replace_range(.., zero_str_with_same_len);
    }
    else if len > zero_str.len() {
        // Or do nothing??

        // Lets, clean only 1st accessible part safely.
        // We can use 'for' but I do not think that we need it for just password cleaning.
        //
        let buf_size = last_char_buf_size_less_than(&str, zero_str.len());
        if let Some(buf_size) = buf_size {
            let zero_str_with_same_len: &str = &zero_str[0..buf_size];
            str.replace_range(0..buf_size, zero_str_with_same_len);
        }
    }
}


fn last_char_buf_size_less_than(str: &str, max_buf_size: usize) -> Option<usize> {
    if str.is_empty() {
        return None;
    }

    if str.len() <= max_buf_size {
        return Some(str.len());
    }

    let mut last_good_index: Option<usize> = None;
    for i in str.char_indices().skip(1) {
        if i.0 <= max_buf_size {
            last_good_index = Some(i.0);
        } else {
            break;
        }
    }
    last_good_index
}


//--------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::{clear_string_chars_impl, last_char_buf_size_less_than};

    #[test]
    fn last_char_buf_size_less_than_test() {
        assert_eq!("Вован".len(), 10);
        assert_eq!(last_char_buf_size_less_than("Вован", 0), None);
        assert_eq!(last_char_buf_size_less_than("Вован", 1), None);
        assert_eq!(last_char_buf_size_less_than("Вован", 2), Some(2));
        assert_eq!(last_char_buf_size_less_than("Вован", 3), Some(2));
        assert_eq!(last_char_buf_size_less_than("Вован", 4), Some(4));
        assert_eq!(last_char_buf_size_less_than("Вован", 5), Some(4));
        assert_eq!(last_char_buf_size_less_than("Вован", 6), Some(6));
        assert_eq!(last_char_buf_size_less_than("Вован", 7), Some(6));
        assert_eq!(last_char_buf_size_less_than("Вован", 8), Some(8));
        assert_eq!(last_char_buf_size_less_than("Вован", 9), Some(8));
        assert_eq!(last_char_buf_size_less_than("Вован", 10), Some(10));
        assert_eq!(last_char_buf_size_less_than("Вован", 11), Some(10));
        assert_eq!(last_char_buf_size_less_than("Вован", 12), Some(10));
        assert_eq!(last_char_buf_size_less_than("Вован", 13), Some(10));
        assert_eq!(last_char_buf_size_less_than("Вован", 14), Some(10));
        assert_eq!(last_char_buf_size_less_than("Вован", 15), Some(10));
    }

    fn call_clear_string_chars_impl(test_str: &str, zero_n: usize) -> String {
        let mut test_str = test_str.to_owned();
        let zero_str = "\0".repeat(zero_n);
        clear_string_chars_impl(&mut test_str, &zero_str);
        test_str
    }

    #[test]
    fn clear_string_chars_impl_test_for_ascii() {
        assert_eq!(call_clear_string_chars_impl("qwerty", 0), "qwerty");
        assert_eq!(call_clear_string_chars_impl("qwerty", 1), "\0werty");
        assert_eq!(call_clear_string_chars_impl("qwerty", 2), "\0\0erty");
        assert_eq!(call_clear_string_chars_impl("qwerty", 3), "\0\0\0rty");
        assert_eq!(call_clear_string_chars_impl("qwerty", 4), "\0\0\0\0ty");
        assert_eq!(call_clear_string_chars_impl("qwerty", 5), "\0\0\0\0\0y");
        assert_eq!(call_clear_string_chars_impl("qwerty", 6), "\0\0\0\0\0\0");
        assert_eq!(call_clear_string_chars_impl("qwerty", 7), "\0\0\0\0\0\0");
        assert_eq!(call_clear_string_chars_impl("qwerty", 8), "\0\0\0\0\0\0");
    }

    #[test]
    fn clear_string_chars_impl_test_for_utf() {
        assert_eq!(call_clear_string_chars_impl("Вован", 0), "Вован");
        assert_eq!(call_clear_string_chars_impl("Вован", 1), "Вован");
        assert_eq!(call_clear_string_chars_impl("Вован", 2), "\0\0ован");
        assert_eq!(call_clear_string_chars_impl("Вован", 3), "\0\0ован");
        assert_eq!(call_clear_string_chars_impl("Вован", 4), "\0\0\0\0ван");
        assert_eq!(call_clear_string_chars_impl("Вован", 5), "\0\0\0\0ван");
        assert_eq!(call_clear_string_chars_impl("Вован", 6), "\0\0\0\0\0\0ан");
        assert_eq!(call_clear_string_chars_impl("Вован", 7), "\0\0\0\0\0\0ан");
        assert_eq!(call_clear_string_chars_impl("Вован", 8), "\0\0\0\0\0\0\0\0н");
        assert_eq!(call_clear_string_chars_impl("Вован", 9), "\0\0\0\0\0\0\0\0н");
        assert_eq!(call_clear_string_chars_impl("Вован", 11), "\0\0\0\0\0\0\0\0\0\0");
        assert_eq!(call_clear_string_chars_impl("Вован", 12), "\0\0\0\0\0\0\0\0\0\0");
        assert_eq!(call_clear_string_chars_impl("Вован", 13), "\0\0\0\0\0\0\0\0\0\0");
        assert_eq!(call_clear_string_chars_impl("Вован", 14), "\0\0\0\0\0\0\0\0\0\0");
        assert_eq!(call_clear_string_chars_impl("Вован", 15), "\0\0\0\0\0\0\0\0\0\0");
    }
}
