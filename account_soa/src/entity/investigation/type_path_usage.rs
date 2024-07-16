

// Just example
#[path = "./../dir1/dir2/some_relative_path_01.rs"]
mod relative_welcome_home;

#[allow(dead_code)]
fn usage_nf_from_relative_path() {
    relative_welcome_home::fn_from_rs_path_01()
}

#[allow(dead_code)]
pub mod usage_test_is_main_src {
    extern crate self as project01;
    pub type TypeAliasByInternalCrateTypePath = mvv_common::backtrace::NewBacktracePolicy;
    pub type TypeAliasByExternCrateTypePath1 = mvv_common::backtrace::NewBacktracePolicy;
    // not compiled
    // pub type TypeAliasByExternCrateTypePath2 = mvv_common::backtrace::NewBacktracePolicy;
}

#[allow(dead_code)]
mod usage_test_is_main_src2 {
    use crate::entity::investigation::type_path_usage::usage_test_is_main_src;

    pub fn temp_fn1() -> usage_test_is_main_src::TypeAliasByInternalCrateTypePath { unimplemented!() }
    pub fn temp_fn2() -> usage_test_is_main_src::TypeAliasByExternCrateTypePath1 { unimplemented!() }
    // pub fn temp_fn3() -> usage_test_is_main_src::TypeAliasByExternCrateTypePath2 { unimplemented!() }
}
