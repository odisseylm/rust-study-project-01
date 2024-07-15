use mvv_common::test::TestOps;

#[ctor::ctor]
static INITIAL_CTOR_2: bool = {
    println!("### INITIAL_CTOR_2");
    true
};


#[static_init::dynamic]
static L1: Vec<i32> = vec![1,2,3,4,5,6];

#[static_init::dynamic(drop)]
static mut L2: Vec<i32> = {let mut v = L1.test_clone(); v.push(43); v};


#[static_init::dynamic]
static L22: Option<& 'static str> = Some(init_value());

fn init_value() -> & 'static str {
    println!("### init_value");
    "Init Value 123"
}


#[test]
fn some_test_to_see_static_is_already_logged() {
    println!("from test some_test_to_see_static_is_already_logged");
}
