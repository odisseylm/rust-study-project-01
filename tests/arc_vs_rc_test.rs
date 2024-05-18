use std::ops::Deref;
use std::time::SystemTime;
use project01::util::TestResultUnwrap;


// const N: i32 = 50_000_000;
const N: i32 = 10_000_000;



#[test]
fn test_01_rc() {
    let v = std::rc::Rc::new(123i64);

    let start = SystemTime::now();

    let mut _sum = 0i64;
    for _ in 1..N {
        _sum += v.deref();
    }

    let end = SystemTime::now();
    println!("spent {:?}", end.duration_since(start).test_unwrap())
}

#[test]
fn test_01_arc() {
    let v = std::sync::Arc::new(123i64);

    let start = SystemTime::now();

    let mut _sum = 0i64;
    for _ in 1..N {
        _sum += v.deref();
    }

    let end = SystemTime::now();
    println!("spent {:?}", end.duration_since(start).test_unwrap());
}


fn test_impl_02<Int: Deref<Target = i64>>(v: &Int) {

    let start = SystemTime::now();

    let mut _sum = 0i64;
    for _ in 1..N {
        _sum += v.deref();
    }

    let end = SystemTime::now();
    println!("spent {:?}", end.duration_since(start).test_unwrap())
}
#[test]
fn test_02_rc()  { test_impl_02(&std::rc::Rc::new(123i64));   }
#[test]
fn test_02_arc() { test_impl_02(&std::sync::Arc::new(123i64)); }



fn test_impl_03<Int: Deref<Target = i64>>(v: &Int) {

    let start = SystemTime::now();

    let mut _sum = 0i64;
    for _ in 1..N {
        _sum = *v.deref();
    }

    let end = SystemTime::now();
    println!("spent {:?}", end.duration_since(start).test_unwrap())
}
#[test]
fn test_03_rc()  { test_impl_03(&std::rc::Rc::new(123i64));   }
#[test]
fn test_03_arc() { test_impl_03(&std::sync::Arc::new(123i64)); }
