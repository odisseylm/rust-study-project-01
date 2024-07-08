
mod common;
mod errors;


use std::error::Error;
use core::fmt::write;
use assertables::{ assert_ge, assert_ge_as_result };
use assertables::{ assert_starts_with, assert_starts_with_as_result };
use assertables::{ assert_contains, assert_contains_as_result };

use project01::util::{BacktraceInfo, enable_backtrace, TestOptionUnwrap, TestResultUnwrap};

use crate::errors::{fn_as_box_error_with_question_op_05, MyError333, MyError334};
use errors::{ extract_json_5, error_fn_5 };
use errors::{ fn_serde_json_wrapped_by_anyhow_using_question_op_05  };
use errors::{ fn_wrap_by_my_error_using_map_err_and_question_op_05  };
use errors::{ fn_wrap_by_my_error_using_map_err_and_with_context_05 };
use errors::{ fn_wrap_by_my_error_using_map_err_05 };
use errors::{ fn_wrap_by_my_error_using_map_err_and_anyhow_macro_05 };
use errors::{ fn_wrap_by_anyhow_error_using_map_err_and_anyhow_macro_05 };
use errors::{ fn_wrap_by_my_error_using_my_fn_to_anyhow_error_fn_05 };
use errors::Entity1;
use project01::util::backtrace::is_anyhow_backtrace_enabled;
use project01::util::test_unwrap::TestSringOps;


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
fn test_str_backtrace() {

    let str_bt = "
       5: error_test::errors::fn_wrap_by_my_error_using_map_err_and_anyhow_macro
             at ./tests/errors/mod.rs:234:5
       6: error_test::errors::fn_wrap_by_my_error_using_map_err_and_anyhow_macro_01
             at ./tests/errors/mod.rs:236:100
       ";

    let bt = BacktraceInfo::from_str(str_bt);

    println!("str backtrace status: {:?}", bt.backtrace_status());
    println!("\n--------------------------------------------------\n");
    println!("str backtrace: {}", bt);
    println!("\n--------------------------------------------------\n");
    println!("str backtrace: {:?}", bt);
}


#[test]
fn test_result_stack_trace() {

    enable_backtrace();

    let v_r: Result<Entity1, serde_json::error::Error> = extract_json_5();
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
    let v_r: Result<Entity1, MyError333> = error_fn_5();
    println!("v1 result: {:?}", v_r);
}



#[test]
fn test_print_error_and_stacktrace_01() {
    enable_backtrace();

    let r = fn_serde_json_wrapped_by_anyhow_using_question_op_05();
    println!("{r:?}");

    println!("\n--------------------------------------------------------------------\n");
    if let Err(err) = fn_serde_json_wrapped_by_anyhow_using_question_op_05() {
        println!("{err:?}");
    }
}

#[test]
fn test_find_my_error_in_error_chain_01() {
    enable_backtrace();
    let r = fn_wrap_by_my_error_using_map_err_and_anyhow_macro_05();
    match r {
        Ok(_) => { assert!(false, "Error is expected.") }
        Err(err) => {
            let my_err: Option<&MyError334> = find_my_error_in_chain(&err);
            assert_eq!(my_err.unwrap().to_test_string(), "Json error 2");
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
    if let Err(err) = fun() {
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
                assert_eq!(my_err.unwrap().to_test_string(), "Json error 2");
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
                assert_eq!(my_err.unwrap().to_test_string(), "missing field `int_field` at line 1 column 48");
            }
        }
    }
}



#[test]
fn test_fn_serde_json_wrapped_by_anyhow_using_question_op() {
    test_impl(fn_serde_json_wrapped_by_anyhow_using_question_op_05, ASSERT_CHAIN_COUNT_EQ_1_ASSERTION | FIND_SERDE_ERROR_ASSERTION);
}

#[test]
fn test_fn_wrap_by_my_error_using_map_err_and_question_op() {
    test_impl(fn_wrap_by_my_error_using_map_err_and_question_op_05, ASSERT_CHAIN_COUNT_EQ_2_ASSERTION | FIND_MY_ERROR_ASSERTION);
}

#[test]
fn test_fn_wrap_by_my_error_using_map_err_and_with_context() {
    test_impl(fn_wrap_by_my_error_using_map_err_and_with_context_05, ASSERT_CHAIN_COUNT_EQ_2_ASSERTION | FIND_MY_ERROR_ASSERTION);
}

#[test]
fn test_fn_wrap_by_my_error_using_map_err() {
    // no chain
    // even no my error!!
    // it just created standard error with MyError.to_test_string() as message
    test_impl(fn_wrap_by_my_error_using_map_err_05, ASSERT_CHAIN_COUNT_EQ_1_ASSERTION);
}

#[test]
fn test_fn_wrap_by_my_error_using_map_err_and_anyhow_macro() {
    test_impl(fn_wrap_by_my_error_using_map_err_and_anyhow_macro_05, ASSERT_CHAIN_COUNT_EQ_1_ASSERTION | FIND_MY_ERROR_ASSERTION);
}

#[test]
fn test_fn_wrap_by_anyhow_error_using_map_err_and_anyhow_macro() {
    test_impl(fn_wrap_by_anyhow_error_using_map_err_and_anyhow_macro_05, ASSERT_CHAIN_COUNT_EQ_1_ASSERTION | FIND_SERDE_ERROR_ASSERTION);
}

#[test]
fn test_fn_wrap_by_my_error_using_my_fn_to_anyhow_error_fn() {
    test_impl(fn_wrap_by_my_error_using_my_fn_to_anyhow_error_fn_05, ASSERT_CHAIN_COUNT_EQ_2_ASSERTION | FIND_MY_ERROR_ASSERTION);
}


#[test]
fn test_result_error_stacktrace_of_anyhow() {
    enable_backtrace();

    let r = fn_wrap_by_my_error_using_map_err_and_with_context_05();
    let err = r.err().test_unwrap();
    println!("err: {err:?}");

    let mut output = String::new();
    write(&mut output, format_args!("{err:?}")).test_unwrap();

    assert_starts_with!(output, "Failed to read/parse json from web.");

    if is_anyhow_backtrace_enabled() {
        assert_contains!(output, "Stack backtrace:");

        assert_contains!(output, "2: error_test::errors::fn_wrap_by_my_error_using_map_err_and_with_context\n             at ./tests/errors/mod.rs:");
        assert_contains!(output, "3: error_test::errors::fn_wrap_by_my_error_using_map_err_and_with_context_01\n             at ./tests/errors/mod.rs:");
        assert_contains!(output, "4: error_test::errors::fn_wrap_by_my_error_using_map_err_and_with_context_02\n             at ./tests/errors/mod.rs:");
        assert_contains!(output, "5: error_test::errors::fn_wrap_by_my_error_using_map_err_and_with_context_03\n             at ./tests/errors/mod.rs:");
        assert_contains!(output, "6: error_test::errors::fn_wrap_by_my_error_using_map_err_and_with_context_04\n             at ./tests/errors/mod.rs:");
        assert_contains!(output, "7: error_test::errors::fn_wrap_by_my_error_using_map_err_and_with_context_05\n             at ./tests/errors/mod.rs:");

        assert_contains!(output, "8: error_test::test_result_error_stacktrace_of_anyhow\n             at ./tests/error_test.rs:");
        // it is risky/dependant
        assert_contains!(output, "9: error_test::test_result_error_stacktrace_of_anyhow::{{closure}}\n             at ./tests/error_test.rs");
    }
}


#[test]
fn test_result_as_box_error_with_question_op_05() {
    enable_backtrace();

    let r = fn_as_box_error_with_question_op_05();
    println!("{:?}", r);

    match r {
        Ok(_) => { assert!(false, "Error is expected") }
        Err(err) => {
            let err_ref = err.as_ref();
            if let Some(my_error) = err_ref.downcast_ref::<MyError334>() {
                println!("### my error: {}", my_error);
                println!("### my error: {:?}", my_error);

                // let err_src = my_error.source();
                if let Some(err_src) = my_error.source() {
                    // println!("### my error src: {}", err_src);
                    println!("### my error src: {:?}", err_src);
                }
            }
            else {
                assert!(false, "MyError334 is not found.")
            }
        }
    }
}
