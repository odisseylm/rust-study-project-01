mod generate_tuple_op_types;
mod util;
mod for_each;
mod feature_cfg;
mod tuple_for_each;
mod tuple_find_some;// RustRover does not pick it up (however cargo does)
// #[macro_use]
// mod compile_log_macros;
include!("./compile_log_macros.rs");

use syn::{ LitInt, parse_macro_input };


// ---------------------------------------------------------------------------------
//                           for-each functions
// ---------------------------------------------------------------------------------

#[proc_macro]
pub fn tuple_for_each_by_ref(params: proc_macro::TokenStream) -> proc_macro::TokenStream {
    tuple_for_each::tuple_for_each_by_ref(params)
}

#[proc_macro]
pub fn tuple_find_some_by_ref(params: proc_macro::TokenStream) -> proc_macro::TokenStream {
    tuple_find_some::tuple_find_some_by_ref(params)
}

#[proc_macro]
pub fn for_each_by_ref(params: proc_macro::TokenStream) -> proc_macro::TokenStream {
    for_each::for_each_by_ref(params)
}


// ---------------------------------------------------------------------------------
//                       Generate tuple types functions
// ---------------------------------------------------------------------------------

/**
 Generated output:
 ```
    pub fn assert_tuple_len_is_0(_tuple: &()) {}
    pub fn assert_tuple_len_is_1<T1>(_tuple: &(T1)) {}
    pub fn assert_tuple_len_is_2<T1,T2>(_tuple: &(T1,T2)) {}
    // ...
 ```
*/
#[proc_macro]
pub fn generate_assert_tuple_len_is(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let max_tuple_len = parse_macro_input!(input as LitInt)
        .base10_parse().unwrap();
    generate_tuple_op_types::generate_assert_tuple_len_is_impl(max_tuple_len).into()
}

#[proc_macro]
pub fn generate_all_tuple_len_traits(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let max_tuple_len = parse_macro_input!(input as LitInt)
        .base10_parse().unwrap();
    generate_tuple_op_types::generate_all_tuple_len_traits(max_tuple_len).into()
}


#[proc_macro]
pub fn generate_all_tuple_access_traits(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let max_tuple_len = parse_macro_input!(input as LitInt)
        .base10_parse().unwrap();
    generate_tuple_op_types::generate_all_tuple_access(max_tuple_len)
}



/*
/* *
 Expected output:
 `` `
    ???
    pub fn const fn tuple_len(_tuple: &()) -> usize { 0 }
    pub fn const fn tuple_len<T1>(_tuple: &(T1)) -> usize { 1 }
    // ...
    pub fn const fn tuple_len<T1,T2,T3>(_tuple: &(T1,T2,T3)) -> usize { 3 }
 `` `
*/
#[proc_macro]
pub fn generate_tuple_len(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let max_tuple_len = parse_macro_input!(input as LitInt);
    let max_tuple_len = max_tuple_len.base10_parse().unwrap();

    generate_assert_tuple_len_is_impl(max_tuple_len).into()
}

fn generate_tuple_len_impl(max_tuple_len: usize) -> proc_macro2::TokenStream {
    ...
}
*/

// -------------------------------------------------------------------------------------------------
//                                        Private tests
// -------------------------------------------------------------------------------------------------


// Tests for private methods/behavior
// Other test are located in ${project}/tests/currency_test.rs
//
#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_of_error_source() {
    }
}
