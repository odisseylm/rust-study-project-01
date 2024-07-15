

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
    use crate::entities::investigation::type_path_usage::usage_test_is_main_src;

    pub fn temp_fn1() -> usage_test_is_main_src::TypeAliasByInternalCrateTypePath { unimplemented!() }
    pub fn temp_fn2() -> usage_test_is_main_src::TypeAliasByExternCrateTypePath1 { unimplemented!() }
    // pub fn temp_fn3() -> usage_test_is_main_src::TypeAliasByExternCrateTypePath2 { unimplemented!() }
}
