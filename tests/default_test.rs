// #![feature(error_generic_meber_access)]

use derivative::Derivative;
use project01::util::backtrace::BacktraceInfo;

#[derive(Debug)]
struct ParamsStruct {
    val1: i32,
    str1: & 'static str,
}

impl Default for ParamsStruct {
    fn default() -> Self {
        ParamsStruct {
            val1: 101,
            str1: "102",
        }
    }
}


#[test]
fn test_all_defaults() {
    println!("test_all_defaults: {:?}", ParamsStruct::default());
    println!("test_all_defaults: {:?}", ParamsStruct { ..ParamsStruct::default()});
}


#[test]
fn test_some_defaults() {
    let s1 = ParamsStruct { val1: 502, ..ParamsStruct::default() };
    println!("test_all_defaults: {:?}", s1);
    // assert_eq!(s1.val1, 101);
    assert_eq!(s1.val1, 502);
    assert_eq!(s1.str1, "102");

    let s2 = ParamsStruct { str1: "503", ..ParamsStruct::default() };
    println!("test_all_defaults: {:?}", s2);
    assert_eq!(s2.val1, 101);
    // assert_eq!(s2.str1, "102");
    assert_eq!(s2.str1, "503");
}


#[test]
fn test_some_defaults_02() {
    let s1 = ParamsStruct { val1: 502, ..Default::default() };
    println!("test_all_defaults: {:?}", s1);
    // assert_eq!(s1.val1, 101);
    assert_eq!(s1.val1, 502);
    assert_eq!(s1.str1, "102");

    let s2 = ParamsStruct { str1: "503", ..Default::default() };
    println!("test_all_defaults: {:?}", s2);
    assert_eq!(s2.val1, 101);
    // assert_eq!(s2.str1, "102");
    assert_eq!(s2.str1, "503");
}


macro_rules! others {
    () => { Default::default() };
}
#[test]
fn test_some_defaults_03() {
    println!("test_all_defaults: {:?}", ParamsStruct { val1: 502, ..others!()});
    println!("test_all_defaults: {:?}", ParamsStruct { str1: "503", ..others!()});
}
#[test]
fn test_some_defaults_03_2() {
    println!("test_all_defaults: {:?}", ParamsStruct { val1: 502, ..others!{}});
    println!("test_all_defaults: {:?}", ParamsStruct { str1: "503", ..others!{}});
}


// macro_rules! Opt {
//     ($t:ty) => {Option<$t>}
// }
// fn asasas() {
//     let x: Option<Vec<i32>> = Some(vec![]);
//     let x: Opt! Vec<i32> = Some(vec![]);
// }


/*
macro_rules! others2 {
    // ($t:ty) => {Default::default()}
    ($t:ty) => (Default::default())
}
#[test]
fn test_some_defaults_04() {
    println!("test_all_defaults: {:?}", ParamsStruct { val1: 502, others2!});
    println!("test_all_defaults: {:?}", ParamsStruct { str1: "503", others2!});
}
*/


/*
macro_rules! others2 {
    () => { ..Default::default() };
}
#[test]
fn test_some_defaults_04() {
    println!("test_all_defaults: {:?}", ParamsStruct { val1: 502, others2!()});
    println!("test_all_defaults: {:?}", ParamsStruct { str1: "503", others2!()});
}
*/

#[derive(Derivative)]
#[derivative(Debug)]
#[allow(dead_code)]
struct Foo {
    foo: u8,
    #[derivative(Debug="ignore")]
    bar: u8,
}

#[test]
fn test_derivative_01() {
    // Prints `Foo { foo: 42 }`
    println!("{:?}", Foo { foo: 42, bar: 1 });
}



/*
trait Add2 {   fn add(v1: i32, v2: i32) -> i32 { v1 + v2 }   }
trait Add3 {   fn add(v1: i32, v2: i32, v3: i32) -> i32 { v1 + v2 + v3 }   }

fn test_add() {
    // use Add2;
    let r1 = add(1, 2); !!! NOT compiled !!!
}
*/


#[derive(Debug, Default)]
struct Struct48374786 {
    f1: i32,
    f2: i32,
}

#[test]
fn test_struct48374786() {
    let s = Struct48374786 {..Default::default()};
    assert_eq!(s.f1, 0);
    assert_eq!(s.f2, 0);
}



#[derive(thiserror::Error, Debug)]
pub enum MyError {
    #[error("My IO error")]
    Io {
        #[from]
        source: std::io::Error,
        // backtrace: std::backtrace::Backtrace,
        // backtrace: BacktraceInfo,
    },
    #[error("My IO error 2")]
    Io2 {
        #[source]
        source: std::io::Error,
        // backtrace: std::backtrace::Backtrace,
        backtrace: BacktraceInfo,
    },
}

/*
impl std::error::Error for MyError {
    fn provide<'a>(&'a self, request: &mut core::error::Request<'a>) {
        request.provide_ref::<BacktraceInfo>(&self.backtrace);
    }
}
*/
