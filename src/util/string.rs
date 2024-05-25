// use const_format::ascii_str;

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
