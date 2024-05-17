use std::ops::Deref;
use std::time::SystemTime;
use project01::util::TestResultUnwrap;

const N: i32 = 10_000_000;




#[test]
fn test_01_rc() {
    let v = std::rc::Rc::new(123i32);

    let start = SystemTime::now();

    let mut sum = 0;
    for _ in 1..N {
        sum += v.deref();
    }

    let end = SystemTime::now();
    println!("spent {:?}", end.duration_since(start).test_unwrap())
}

#[test]
fn test_01_arc() {
    let v = std::sync::Arc::new(123i32);

    let start = SystemTime::now();

    let mut sum = 0;
    for _ in 1..N {
        sum += v.deref();
    }

    let end = SystemTime::now();
    println!("spent {:?}", end.duration_since(start).test_unwrap());
}


fn test_impl_02<Int: Deref<Target = i32>>(v: &Int) {

    let start = SystemTime::now();

    let mut sum = 0;
    for _ in 1..N {
        sum += v.deref();
    }

    let end = SystemTime::now();
    println!("spent {:?}", end.duration_since(start).test_unwrap())
}
#[test]
fn test_02_rc()  { test_impl_02(&std::rc::Rc::new(123i32));   }
#[test]
fn test_02_arc() { test_impl_02(&std::sync::Arc::new(123i32)); }



fn test_impl_03<Int: Deref<Target = i32>>(v: &Int) {

    let start = SystemTime::now();

    let mut sum = 0;
    for _ in 1..N {
        sum = *v.deref();
    }

    let end = SystemTime::now();
    println!("spent {:?}", end.duration_since(start).test_unwrap())
}
#[test]
fn test_03_rc()  { test_impl_02(&std::rc::Rc::new(123i32));   }
#[test]
fn test_03_arc() { test_impl_02(&std::sync::Arc::new(123i32)); }

