use assertables::{ assert_ge, assert_ge_as_result };

use project01::util::{enable_backtrace, MyError333, MyError334};
#[allow(unused_imports)]
use project01::util::{ error_fn_105, error_fn_205, error_fn_305, error_fn_405, error_fn_505, error_fn_605, error_fn_705 };
use project01::util::Entity1;

mod common;

// #[derive(Deserialize)]
// struct ClusterMap {
//     int_value: i32,
//     field: String,
// }


#[test]
fn test_print_current_stack_trace() {
    enable_backtrace();
    project01::util::backtrace::print_current_stack_trace();
}


#[test]
fn test_result_stack_trace() {

    enable_backtrace();

    let v_r: Result<Entity1, serde_json::error::Error> = project01::util::extract_json_5();
    println!("v1 result: {:?}", v_r);

    match v_r {
        Ok(v) => {
            println!("v1: {:?}", v);
            assert_eq!(v.int_field, 12322);
        }
        Err(err) => {
            println!("v1 result err: {:?}", err);
            println!("v1 result err: {:?}", err);
        }
    }
}


#[test]
fn test_result_stack_trace333() {
    enable_backtrace();
    let v_r: Result<Entity1, MyError333> = project01::util::error_fn_5();
    println!("v1 result: {:?}", v_r);
}



#[test]
fn test_print_error_and_stacktrace_01() {
    enable_backtrace();
    let r = error_fn_105();
    println!("{r:?}");
}

#[test]
fn test_print_error_and_stacktrace_02() {
    enable_backtrace();
    if let Err(err) = error_fn_105() {
        println!("{err:?}");
    }
}

#[test]
fn test_find_my_error_in_error_chain_01() {
    enable_backtrace();
    let r = error_fn_505();
    match r {
        Ok(_) => { assert!(false, "Error is expected.") }
        Err(err) => {
            let my_err: Option<&MyError334> = find_my_error_in_chain(&err);
            assert_eq!(my_err.unwrap().to_string(), "Json error 2");
        }
    }
}

fn find_my_error_in_chain(error: &anyhow::Error) -> Option<&MyError334> {
    let mut i = 0;
    for cause in error.chain() {
        i += 1;
        println!("cause {}: {cause:?}", i);
        if let Some(my_error) = cause.downcast_ref::<MyError334>() {
            println!("### my error");
            let temp1: &MyError334 = my_error;
            return Some(temp1);
        }
    }
    None
}
fn find_serde_error_in_chain(error: &anyhow::Error) -> Option<&serde_json::Error> {
    let mut i = 0;
    for cause in error.chain() {
        i += 1;
        println!("cause {}: {cause:?}", i);
        if let Some(serde_error) = cause.downcast_ref::<serde_json::Error>() {
            println!("### serde error");
            let temp1: &serde_json::Error = serde_error;
            return Some(temp1);
        }
    }
    None
}

const ASSERT_CHAIN_COUNT_EQ_1_ASSERTION: i32 = 1;
const ASSERT_CHAIN_COUNT_EQ_2_ASSERTION: i32 = 1<<1;
const FIND_MY_ERROR_ASSERTION: i32 = 1<<2;
const FIND_SERDE_ERROR_ASSERTION: i32 = 1<<3;

fn test_impl<Ok: core::fmt::Debug>(fun: fn()->Result<Ok,anyhow::Error>, test_steps: i32) {
    let r = fun();

    println!("{r:?}");
    if let Err(err) = error_fn_105() {
        println!("{err:?}");
    }

    if test_steps & (ASSERT_CHAIN_COUNT_EQ_1_ASSERTION | ASSERT_CHAIN_COUNT_EQ_2_ASSERTION) != 0 {
        if let Err(error) = r {
            let mut count = 0;

            let mut i = 0;
            for cause in error.chain() {
                i += 1;
                println!("cause {}: {cause:?}", i);
                count += 1;

                if let Some(_) = cause.downcast_ref::<std::io::Error>() {
                    println!("IO Error found");
                }
                if let Some(_) = cause.downcast_ref::<MyError334>() {
                    println!("JsonError2 found");
                }
            }

            if test_steps & (ASSERT_CHAIN_COUNT_EQ_1_ASSERTION) != 0 {
                assert_ge!(count, 1);
            }
            else if test_steps & (ASSERT_CHAIN_COUNT_EQ_2_ASSERTION) != 0 {
                assert_ge!(count, 2);
            }
            else {
                assert!(false, "Unexpected ASSERT_CHAIN_COUNT_EQ_XXX_ASSERTION");
            }
        } else {
            assert!(false, "Error is expected.")
        }
    }

    if test_steps & FIND_MY_ERROR_ASSERTION != 0 {
        let r = fun();
        match r {
            Ok(_) => { assert!(false, "Error is expected.") }
            Err(err) => {
                let my_err: Option<&MyError334> = find_my_error_in_chain(&err);
                assert!(my_err.is_some(), "Error MyError334 is not found.");
                assert_eq!(my_err.unwrap().to_string(), "Json error 2");
            }
        }
    }

    if test_steps & FIND_SERDE_ERROR_ASSERTION != 0 {
        let r = fun();
        match r {
            Ok(_) => { assert!(false, "Error is expected.") }
            Err(err) => {
                let my_err: Option<&serde_json::Error> = find_serde_error_in_chain(&err);
                assert!(my_err.is_some(), "Error serde_json::Error is not found.");
                assert_eq!(my_err.unwrap().to_string(), "missing field `int_field` at line 1 column 48");
            }
        }
    }
}



#[test]
fn test_fn_error_fn_105() {
    test_impl(error_fn_105, ASSERT_CHAIN_COUNT_EQ_1_ASSERTION | FIND_SERDE_ERROR_ASSERTION);
}

#[test]
fn test_fn_error_fn_205() {
    test_impl(error_fn_205, ASSERT_CHAIN_COUNT_EQ_2_ASSERTION | FIND_MY_ERROR_ASSERTION);
}

#[test]
fn test_fn_error_fn_305() {
    test_impl(error_fn_305, ASSERT_CHAIN_COUNT_EQ_2_ASSERTION | FIND_MY_ERROR_ASSERTION);
}

#[test]
fn test_fn_error_fn_405() {
    // no chain
    // even no my error!!
    // it just created standard error with MyError.to_string() as message
    test_impl(error_fn_405, ASSERT_CHAIN_COUNT_EQ_1_ASSERTION);
}

#[test]
fn test_fn_error_fn_505() {
    test_impl(error_fn_505, ASSERT_CHAIN_COUNT_EQ_1_ASSERTION | FIND_MY_ERROR_ASSERTION);
}

#[test]
fn test_fn_error_fn_605() {
    test_impl(error_fn_605, ASSERT_CHAIN_COUNT_EQ_1_ASSERTION | FIND_SERDE_ERROR_ASSERTION);
}

#[test]
fn test_fn_error_fn_705() {
    test_impl(error_fn_705, ASSERT_CHAIN_COUNT_EQ_2_ASSERTION | FIND_MY_ERROR_ASSERTION);
}
