mod common;
use common::TestResultUnwrap;
use core::fmt;
//--------------------------------------------------------------------------------------------------


struct Position {
    longitude: f32,
    latitude: f32,
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.longitude, self.latitude)
    }
}


// custom Debug impl
impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("")
            .field(&self.longitude)
            .field(&self.latitude)
            .finish()
    }
}


#[test]
fn test_format_usage_01() {
    assert_eq!("(1.987, 2.983)",
               format!("{}", Position { longitude: 1.987, latitude: 2.983, }));
}


#[test]
fn test_format_usage_02() {
    let position = Position { longitude: 1.987, latitude: 2.983 };
    assert_eq!(format!("{position:?}"), "(1.987, 2.983)");

    // #? - pretty-print the Debug formatting (adds linebreaks and indentation)
    assert_eq!(format!("{position:#?}"), "(
    1.987,
    2.983,
)");
}


#[test]
fn test_format_usage_03() {
    let position = Position { longitude: 1.987, latitude: 2.983 };
    assert_eq!(format!("{position:?}"), "(1.987, 2.983)");

    assert_eq!(format!("{position}"), "(1.987, 2.983)");
}


// !!! It seems like hanged up test and have a bit broken output under RustRover (and probably Idea)
// !!! It happens only if no '\n' in string printed by write(bytes) method !!??
// !!! From command line its output looks OK.
#[test]
fn test_format_usage_04() {
    use core::fmt::write;
    use std::io::Write;

    println!("### test_format_usage_04");

    let mut w = Vec::new();
    write!(&mut w, "Hello {}!\n\n", "world").test_unwrap();

    let mut output = String::new();
    // if let Err(fmt::Error) = write(&mut output, format_args!("Hello {}!\n", "world\n")) {
    //     panic!("An error occurred");
    // }

    // wit this we have 'hangs up like' behavior
    // write(&mut output, format_args!("Hello {}!", "world\n")).test_unwrap();

    write(&mut output, format_args!("Hello {}!\n", "world\n")).test_unwrap();

    let str_ref = output.as_str();
    println!("str_ref 01: {str_ref}");
    println!("str_ref 02: {str_ref:}");
    println!("str_ref 03: {str_ref:?}");

    let buf: &[u8] = output.as_str().as_bytes();
    println!("buf: {buf:?}");
    println!("buf size: {}", buf.len());

    std::io::stdout().write(buf).test_unwrap();
    std::io::stdout().flush().test_unwrap();

    // No '\n' in print! does not cause 'hangs up like' behavior.
    print!(" apple ");
    print!(" fruit ");
}


// !!! It seems like hanged up test and have a bit broken output under RustRover (and probably Idea)
// !!! It happens only if no '\n' in string printed by write(bytes) method !!??
// !!! From command line its output looks OK.
#[test]
fn test_looks_like_hanged_up() {
    use std::io::{ Write };

    println!("### test_looks_like_hanged_up");

    let mut some_writer = std::io::stdout();

    // with this line without '\n' looks like hanged up under RustRover !?
    // write!(&mut some_writer, "{}", format_args!("print with a {}", "macro")).test_unwrap();

    write!(&mut some_writer, "{}", format_args!("print with a {}\n", "macro")).test_unwrap();
    some_writer.flush().test_unwrap();
}


#[test]
fn test_looks_like_hanged_up_54545() {
    use core::fmt::write;

    let mut some_writer = String::new();
    write(&mut some_writer, format_args!("Hello {}!", "world\n")).test_unwrap();
    println!("{some_writer}");
}
