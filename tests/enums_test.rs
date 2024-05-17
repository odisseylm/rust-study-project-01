// #![feature(offset_of_enum, offset_of_nested)]

// C like enum
// #[repr(u8)]
// #[repr(u32)] // long C type
// #[repr(core::ffi::c_int)] // long C type TODO: not compiled
// #[repr(core::ffi::c_int)] // long C type TODO: not compiled
#[repr(C)] // it is working
#[repr(C)] // long C type TODO: not compiled
#[allow(dead_code)]
enum CEnum01 {
    Val1 = 1,
    Val2 = 2,
}


#[allow(dead_code)]

struct Abcdef {
    f1: core::ffi::c_int,
    f2: core::ffi::c_long,
    f3: core::ffi::c_ulonglong,
}


#[repr(u8)] // T O D O: Why it successfully compiled? How does it work?
#[allow(dead_code)]
enum CEnum2 {
    Unit = 3,
    IntVal(u16),
    Tuple(u16),
    Struct {
        a: u8,
        b: u16,
        c: u32,
    } = 1,
}

fn aa() {
    // let v: u32 = unsafe { CEnum01::Val1 }::;
    let _v1: u32 = CEnum01::Val1 as u32;
    // let _v2: u8 = CEnum2::Unit as u8;
    // let _v2: u32 = CEnum2::Struct as u32;
}

#[allow(dead_code)]
enum RustStdEnum {
    Unit,
    IntVal(u16),
    Tuple(u16),
    Struct {
        a: u8,
        b: u16,
        c: u32,
    },
}


// #[test]
// fn test_enum_count() {
//     println!("CEnum2 variant count: {}", std::mem::variant_count::<CEnum2>()) // !!! unstable !!!
// }

//noinspection ALL,RsUnresolvedPath
#[test]
fn test_ggfgfg() {
    // core::ffi::

    println!("size of CEnum01: {} bytes", core::mem::size_of::<CEnum01>());

    println!("size of CEnum2: {} bytes", core::mem::size_of::<CEnum2>());
    println!("size of CEnum2::Unit: {} bytes", core::mem::size_of_val(&CEnum2::Unit));
    println!("size of CEnum2::IntVal: {} bytes", core::mem::size_of_val(&CEnum2::IntVal));
    println!("size of CEnum2::Tuple: {} bytes", core::mem::size_of_val(&CEnum2::Tuple));

    // println!("size of CEnum2::Struct: {} bytes", core::mem::size_of_val(CEnum2::Struct));
    // println!("size of CEnum2::Struct: {} bytes", core::mem::size_of_val(CEnum2::Struct));
    // let s = CEnum2::Struct;
    // println!("size of CEnum2::Struct: {} bytes", core::mem::size_of_val(&s));

    // println!("size of CEnum2::Struct: {} bytes", core::mem::size_of::<CEnum2::Struct>());

    println!("size of RustStdEnum: {} bytes", core::mem::size_of::<RustStdEnum>());
    println!("size of RustStdEnum::Unit: {} bytes", core::mem::size_of_val(&RustStdEnum::Unit));
    println!("size of RustStdEnum::IntVal: {} bytes", core::mem::size_of_val(&RustStdEnum::IntVal));
    println!("size of RustStdEnum::Tuple: {} bytes", core::mem::size_of_val(&RustStdEnum::Tuple));

    // println!("size of RustStdEnum::Struct: {} bytes", core::mem::size_of_val(RustStdEnum::Struct));
    // println!("size of RustStdEnum::Struct: {} bytes", core::mem::size_of_val(&RustStdEnum::Struct));
    // let s = RustStdEnum::Struct;
    // println!("size of RustStdEnum::Struct: {} bytes", core::mem::size_of_val(&s));

    // println!("size of CEnum2::Struct: {} bytes", core::mem::size_of::<CEnum2::Struct>());

    println!("size of Struct1: {} bytes", core::mem::size_of::<Struct1>());
    let struct1 = Struct1 { ..Default::default() };
    println!("size of struct1: {} bytes", core::mem::size_of_val(&struct1));

    let v_i64 = 1i64;
    println!("size of v_i64: {} bytes", core::mem::size_of_val(&v_i64));
}


#[repr(C)] // without this, rust changes (optimizes) orders of struct fields
#[derive(Default)]
#[allow(dead_code)]
struct Struct1 {
    f1_i8: i8,
    f2_i64: i64,
    f3_i32: i32,
}


//noinspection ALL
#[test]
fn test_offset() {
    //noinspection ALL
    let f1_i8_offset  = core::mem::offset_of!(Struct1, f1_i8);
    let f2_i64_offset = core::mem::offset_of!(Struct1, f2_i64);
    let f3_i32_offset = core::mem::offset_of!(Struct1, f3_i32);

    println!("offset of f1_i8  is {} bytes", f1_i8_offset);
    println!("offset of f2_i64 is {} bytes", f2_i64_offset);
    println!("offset of f3_i32 is {} bytes", f3_i32_offset);
}


/*
use std::mem::size_of;

enum Foo {
    Cons(~char)
}

enum Bar {
    Cons(~char),
    Nil
}

println!("{}", size_of::<Foo>());
println!("{}", size_of::<Bar>());
*/
