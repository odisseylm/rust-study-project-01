

/*
// extern crate static_assertions as sa;
// extern crate const_format as cf;
// use const_format::concatcp;

fn test4786378463784637826478364783() {
    const PASSWORD: &str = "password";

    const fn times() -> u64 { 10 }

    const MSG: &str =
        const_format::concatcp!("The password is \"", PASSWORD, "\", you can only guess ", times(), " times.");

    assert_eq!(MSG, r#"The password is "password", you can only guess 10 times."#);
}
*/


/*

// https://users.rust-lang.org/t/concatenate-const-strings/51712/10

macro_rules! combine {
    ($A:expr, $B:expr) => {{
        const LEN: usize = $A.len() + $B.len();
        const fn combine(a: &'static str, b: &'static str) -> [u8; LEN] {
            let mut out = [0u8; LEN];
            out = copy_slice(a.as_bytes(), out, 0);
            out = copy_slice(b.as_bytes(), out, a.len());
            out
        }
        const fn copy_slice(input: &[u8], mut output: [u8; LEN], offset: usize) -> [u8; LEN] {
            let mut index = 0;
            loop {
                output[offset+index] = input[index];
                index += 1;
                if index == input.len() { break }
            }
            output
        }
        combine($A, $B)
    }}
}

const BASE: &'static str = "path/to";
const PART: &'static str = "foo";
const PATH: &'static [u8] = &combine!(BASE, PART);

fn main() {
    // Once you're confident it's working you can use `from_utf8_unchecked` here.
    let s = core::str::from_utf8(PATH).expect("Something went badly wrong at compile time.");
    dbg!(s);
}

*/


/*
macro_rules! c_c_combo {
    ( $combined:ident : $($name:ident : $type:ty = $value:expr);+ $(;)? ) => {
        $(
            const $name: $type = $value;
        )+
        const $combined: &'static str = concat!($($value),+);
    }
}

c_c_combo! (PATH:
    BASE: &'static str = "path/to";
    PART: &'static str = "foo";
);

fn main() {
    println!("{}", BASE);
    println!("{}", PART);
    println!("{}", PATH);
}
*/


/*
const VERSION_STRING: &'static str =
    concat!("my program v", env!("CARGO_PKG_VERSION"));
*/

/*
use const_format::concatcp;

const DESCRIPTION: &'static str = "my program";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");
const VERSION_STRING: &'static str = concatcp!(DESCRIPTION, " v", VERSION);
*/