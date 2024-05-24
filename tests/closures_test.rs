use std::thread::spawn;
use project01::util::TestResultUnwrap;



fn fn_returns_void() { }
#[test]
fn fn_92() {
    let f1: fn() = fn_returns_void;
    f1();

    let f1: fn()->() = fn_returns_void;
    f1();

    // let f2: i32 = fn_returns_void;
}


fn fn_returns_i32(v: i32) -> i64 { v as i64 }
#[test]
fn fn_93() {
    let f1: fn(i32)->i64 = fn_returns_i32;
    f1(234);

    // let f2: i32 = fn_returns_void;
}


#[test]
fn test_move_closure() {
    let v = vec![1,2,3];

    // let f = move || { println!("{:?}", v) };
    let f: Box<dyn FnOnce() + Send + 'static> = Box::new( move || { println!("{:?}", v) } );

    // drop(f);
    // drop(String::new());

    struct S1 { _x: i32 }
    let s1 = S1 { _x: 123 };
    drop(s1);

    // drop(123);

    spawn(f).join().test_unwrap();

    // let closure: dyn FnOnce()->() = move || { println!("{:?}", v) };
    // spawn(closure).join().test_unwrap();
}
