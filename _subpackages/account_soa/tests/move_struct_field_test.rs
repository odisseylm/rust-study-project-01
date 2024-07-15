
#[derive(Debug)]
struct S1 {
    #[allow(dead_code)]
    s_f: String,
    bt_f: std::backtrace::Backtrace,
}

// #[derive(Debug)]
struct S2 {
    #[allow(dead_code)]
    s_f: String,
    bt_f: std::cell::Cell<std::backtrace::Backtrace>,
}

impl core::fmt::Debug for S2 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "S2 {{ s_f: {:?}, bt_f: {:?} }}", self.s_f, self.bt_f.as_ptr())
    }
}


impl S2 {
    fn move_bt(&self) -> std::backtrace::Backtrace {
        let bt_moved: std::backtrace::Backtrace = self.bt_f.replace(std::backtrace::Backtrace::disabled());
        bt_moved
    }
}


#[test]
fn test_moving_mut_struct_field() {

    let mut s1 = S1 {
        s_f: "1".to_test_string(),
        bt_f: std::backtrace::Backtrace::force_capture(),
    };
    println!("s: {:?}", s1);

    let bt = s1.bt_f;
    println!("bt: {:?}", bt);

    // println!("s: {:?}", s1);

    s1.bt_f = std::backtrace::Backtrace::disabled();
    println!("s: {:?}", s1);
}


#[test]
fn test_moving_struct_field() {

    let s1 = S2 {
        s_f: "1".to_test_string(),
        bt_f: std::cell::Cell::new(std::backtrace::Backtrace::force_capture()),
    };
    println!("s: {:?}", s1);

    let bt = s1.move_bt();
    println!("bt: {:?}", bt);
    println!("s: {:?}", s1);
}


/*
pub enum Cow<'a, B> where B: 'a + ToOwned + ?Sized {
    Borrowed(&'a B),
    Owned(<B as ToOwned>::Owned),
}

impl<'a> Into<Cow<'a, str>> for & 'a str {
    fn into(self) -> Cow<'a, str> { Cow::Borrowed::<'a, str>(&self) }
}
impl<'a> Into<Cow<'a, str>> for String {
    fn into(self) -> Cow<'a, str> { Cow::Owned(self) }
}
*/


use std::borrow::Cow;
use mvv_common::test::TestSringOps;

#[test]
fn cow_test() {
    let s: String = "qwerty".to_test_string();
    println!("s: {}", s);

    let b = Cow::Borrowed(&s);
    if let Cow::Borrowed(v) = b { println!("Borrowed: {}", v) }
    println!("s: {}", s);

    let b: Cow<'_, String> = Cow::Owned(s);
    if let Cow::Owned(v) = b { println!("Owned: {}", v) }
    // println!("s: {}", s);

    // assert!(false, "To see output");
}


fn _modulo_3(input: u8) -> Cow<'static, str> {
    match input % 3 {
        0 => "Remainder is 0".into(),
        1 => "Remainder is 1".into(),
        remainder => format!("Remainder is {}", remainder).into(),
    }
}


#[test]
fn move_to_slice_test() {

    let string = "qwerty";
    println!("### string: {}", string);

    // let str = string[0..2];
    let str: &str = &string[0..2];
    println!("### str: {}", str);

    println!("### string: {}", string);
}

fn _aaa(string: String) -> Box<str> {
    string.into()
}
