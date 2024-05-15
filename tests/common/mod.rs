#[allow(unused_imports)]
pub use project01::util::test_unwrap::{TestOptionUnwrap, TestResultUnwrap};


#[allow(dead_code)]
pub fn setup() {
    // setup code specific to your library's tests would go here
    println!("### setup()")
}


// Seems it does not work.
//use ctor::ctor;
// #[ctor]
// fn init_color_backtrace() {
//     color_backtrace::install();
// }
