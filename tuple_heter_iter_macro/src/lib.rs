
// RustRover does not pick it up (however cargo does)
// #[macro_use]
// mod compile_log_macros;
include!("./compile_log_macros.rs");


use itertools::*;

use quote::{ quote, /*TokenStreamExt,*/ ToTokens };
use syn::{ LitInt, parse_macro_input };
// use syn::spanned::Spanned;


const MAX_TUPLE_LEN: usize =
if cfg!(feature = "tuple_len_64") {
    64
} else if cfg!(feature = "tuple_len_32") {
    32
} else {
    16
};


#[proc_macro]
pub fn for_each_by_ref(params: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let mut params_vec = split_params(params);

    assert!(params_vec.len() >= 2, "Expected at least one param and run block.");

    let for_each_run_block = params_vec.pop()
        .expect("Expected at least one param and run block.");

    let for_each_code = quote! {
        #(
             let item_ref = & (#params_vec);
             #for_each_run_block
        )*
    };

    let out: proc_macro::TokenStream = for_each_code.into();
    out
}


#[proc_macro]
pub fn for_each_in_tuple_by_ref(params: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let mut params_vec = split_params(params);

    assert_eq!(params_vec.len(), 3, "Expected ???.");

    let for_each_run_block = params_vec.pop()
        .expect("Expected ???.");

    let tuple = params_vec.get(0)
        .expect("Expected ???.");
    let tuple_len_ts = params_vec.get(1);
    eprintln!("### tuple count: {:?}", tuple_len_ts);

    let count: Option<usize> = as_uint_literal(tuple_len_ts.map(|v|v.clone()));
    let count = count.expect("Expects tuple size as usize literal");

    let mut vars = Vec::<proc_macro2::TokenStream>::new();
    for i in 0..count {
        vars.push( quote!( ((#tuple).#i) ) );
    }

    let for_each_code = quote! {
        #(
             let item_ref = & (#vars);
             #for_each_run_block
        )*
    };

    let out: proc_macro::TokenStream = for_each_code.into();
    out
}



fn split_params(params: proc_macro::TokenStream) -> Vec<proc_macro2::TokenStream> {
    use proc_macro2::TokenTree;
    let params2: proc_macro2::TokenStream = params.into();

    let mut params_vec = Vec::<proc_macro2::TokenStream>::new();
    let mut current_param_as_stream = proc_macro2::TokenStream::new();

    use itertools::Itertools;

    for (pos, tt) in params2.into_iter().with_position() {

        let mut end_of_func_param = false;
        use quote::TokenStreamExt;

        if let TokenTree::Punct(ref punct) = tt {
            if punct.as_char() == ',' {
                end_of_func_param = true;
            }
            else {
                current_param_as_stream.append(tt);
            }
        } else {
            current_param_as_stream.append(tt);
        }

        if let Position::Last | Position::Only = pos {
            end_of_func_param = true;
        }

        if end_of_func_param {
            params_vec.push(current_param_as_stream);
            current_param_as_stream = proc_macro2::TokenStream::new();
        }
    }

    params_vec
}


fn as_uint_literal(token_stream: Option<proc_macro2::TokenStream>) -> Option<usize> {
    use proc_macro2::TokenTree;

    for tt in token_stream.unwrap().into_iter() {
        return match tt {
            TokenTree::Literal(ref lit) => {
                let lit_str = lit.to_string();
                let as_uint: Option<usize> = core::str::FromStr::from_str(lit_str.as_str()).ok();
                as_uint
            }
            _ => None,
        }
    }
    None
}



/**
 Expected output:
 ```
    pub fn assert_tuple_len_is_0(_tuple: &()) {}
    pub fn assert_tuple_len_is_1<T1>(_tuple: &(T1)) {}
    pub fn assert_tuple_len_is_2<T1,T2>(_tuple: &(T1,T2)) {}
    pub fn assert_tuple_len_is_3<T1,T2,T3>(_tuple: &(T1,T2,T3)) {}
    pub fn assert_tuple_len_is_4<T1,T2,T3,T4>(_tuple: &(T1,T2,T3,T4)) {}
 ```
*/
#[proc_macro]
pub fn generate_assert_tuple_len_is(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let max_tuple_len = parse_macro_input!(input as LitInt);
    let max_tuple_len = max_tuple_len.base10_parse().unwrap();

    generate_assert_tuple_len_is_impl(max_tuple_len).into()
}

fn generate_assert_tuple_len_is_impl(max_tuple_len: usize) -> proc_macro2::TokenStream {

    let assert_tuple_len_is_functions = (1..max_tuple_len)
        .into_iter()
        .map(|i|{
            let method_ident = make_ident(format!("assert_tuple_len_is_{i}"));
            let types = types_list(i);
            quote! {
                pub fn #method_ident < #(#types),* >(_tuple: &(#(#types),*)) {}
            }
        })
        .collect::<Vec<_>>();

    let q = quote! {
        pub fn assert_tuple_len_is_0(_tuple: &()) {}
        #(#assert_tuple_len_is_functions)*
    };
    q.into()
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

#[proc_macro]
pub fn generate_all_tuple_ops(input: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let max_tuple_len = parse_macro_input!(input as LitInt);
    let max_tuple_len = max_tuple_len.base10_parse().unwrap();

    let trait_def = generate_tuple_ops_trait_impl(max_tuple_len);

    let impls =  (1..max_tuple_len)
        .into_iter()
        .map(|tuple_len| generate_tuple_ops_impl(max_tuple_len, tuple_len))
        .collect::<Vec<_>>();

    let assert_len_functions = generate_assert_tuple_len_is_impl(max_tuple_len);

    let out_ps2: proc_macro2::TokenStream = quote! {
        #trait_def
        #(#impls)*

        #assert_len_functions
    };
    out_ps2.into()
}


#[proc_macro]
pub fn generate_tuple_ops_trait(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let max_tuple_len = parse_macro_input!(input as LitInt);
    let max_tuple_len = max_tuple_len.base10_parse().unwrap();
    generate_tuple_ops_trait_impl(max_tuple_len).into()
}
/**
Generates code like
```
pub trait TupleOps {
    const LENGTH: usize;
    fn tuple_len(&self) -> usize { Self::LENGTH }
    // ?? Can we safely use such short name ??
    fn len(&self) -> usize { Self::LENGTH }
    type Elem0;
    fn _0(&self) -> Option<&Self::Elem0>;
    type Elem1;
    fn _1(&self) -> Option<&Self::Elem1>;
    type Elem2;
    fn _2(&self) -> Option<&Self::Elem2>;
    type Elem3;
    fn _3(&self) -> Option<&Self::Elem3>;
    type Elem4;
    fn _4(&self) -> Option<&Self::Elem4>;
}
```
*/
fn generate_tuple_ops_trait_impl(max_tuple_len: usize) -> proc_macro2::TokenStream {
    use proc_macro2::TokenStream as PM2TS;

    let rows: Vec<PM2TS> = (0..max_tuple_len)
        .into_iter()
        .map(|i|{
            let elem_type_ident = make_ident(format!("Elem{i}"));
            let method_ident = make_ident(format!("_{i}"));

            quote! {
                type #elem_type_ident;
                fn #method_ident(&self) -> Option<&Self:: #elem_type_ident>;
            }
        })
        .collect();

    let out: PM2TS = quote!{
        pub trait TupleOps {
            const LENGTH: usize;
            fn tuple_len(&self) -> usize { Self::LENGTH }
            // ?? Can we safely use such short name ??
            fn len(&self) -> usize { Self::LENGTH }
            #(#rows)*
        }
    };
    out.into()
}


/**
Generates code like
``
impl <T0,T1> TupleOps for (T0,T1) {
    type Elem0 = T0;
    #[inline(always)]
    fn _0(&self) -> Option<&Self::Elem0> { Some(&self.0) }

    type Elem1 = T1;
    #[inline(always)]
    fn _1(&self) -> Option<&Self::Elem1> { Some(&self.1) }

    // Not supported
    type Elem2 = T0;
    #[inline(always)]
    fn _2(&self) -> Option<&Self::Elem02> { None }
}
``
*/
fn generate_tuple_ops_impl(max_tuple_len: usize, current_tuple_len: usize)
                           -> proc_macro2::TokenStream {
    use proc_macro2 as pm2;
    use proc_macro2::TokenStream as PM2TS;

    let current_tuple_len_literal = pm2::TokenTree::Literal(
        pm2::Literal::usize_unsuffixed(current_tuple_len));

    let types = types_list(current_tuple_len);

    let matched_type_rows: Vec<pm2::TokenStream> = (0..current_tuple_len)
        .into_iter()
        .map(|i| {
            // let index = proc_macro2::TokenTree::Literal(proc_macro2::Literal::usize_unsuffixed(i));
            let index = syn::Index::from(i);
            let gen_elem_type_ident = make_ident(format!("T{i}"));
            let elem_type_ident = make_ident(format!("Elem{i}"));
            let method_ident = make_ident(format!("_{i}"));

            quote! {
                type #elem_type_ident = #gen_elem_type_ident;
                #[inline(always)]
                fn #method_ident(&self) -> Option<&Self:: #elem_type_ident> { Some(&self. #index) }
            }
        })
        .collect::<Vec<_>>();


    let unmatched_type_rows =  (current_tuple_len..max_tuple_len)
        .into_iter()
        .map(|i| {
            let elem_type_ident = make_ident(format!("Elem{i}"));
            let method_ident = make_ident(format!("_{i}"));

            quote! {
                type #elem_type_ident = T0;
                #[inline(always)]
                fn #method_ident(&self) -> Option<&Self:: #elem_type_ident> { None }
            }
        })
        .collect::<Vec<_>>();


    let out: PM2TS = quote! {
        impl < #(#types),* > TupleOps for ( #(#types),* ,) {
            const LENGTH: usize = #current_tuple_len_literal;
            #[inline(always)]
            fn tuple_len(&self) -> usize { #current_tuple_len_literal }
            // ?? Can we safely use such short name ??
            #[inline(always)]
            fn len(&self) -> usize { #current_tuple_len_literal }
            #(#matched_type_rows)*
            #(#unmatched_type_rows)*
        }
    };
    out.into()
}

fn make_ident(ident: String)
    -> proc_macro2::TokenStream
{
    // proc_macro2::TokenTree::Ident(proc_macro2::Ident::new(ident.as_str(), quote!{}.span()))

    // or
    let ident: syn::Ident = syn::parse_str(ident.as_str())
        .expect(&format!("Error of converting \"{ident}\" to Ident."));
    ident.into_token_stream()
}

/**
 * Generates types quote like 'T0,T1,T2...'
 */
fn types_list(type_count: usize) -> Vec<proc_macro2::TokenStream> {
    (0..type_count)
        .into_iter()
        .map(|i| make_ident(format!("T{i}")))
        .collect::<Vec<_>>()
}

#[proc_macro]
pub fn for_each_in_tuple_by_ref_2(params: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let mut params_vec = split_params(params);

    // const input_err_msg = "Input should be { tuple_expr, { code } }.";
    macro_rules! input_err_msg { () => { "Input should be {{ tuple_expr, {{ code }} }}." }; }

    assert_eq!(params_vec.len(), 2, input_err_msg!());

    let for_each_run_block = params_vec.pop().expect(input_err_msg!());
    let tuple = params_vec.get(0).expect(input_err_msg!());

    let max_tuple_len = MAX_TUPLE_LEN;

    let mut vars = Vec::<proc_macro2::TokenStream>::new();
    for i in 0..max_tuple_len {
        let method_ident = make_ident(format!("_{i}"));
        vars.push( quote!( ((#tuple).#method_ident()) ) );
    }

    let for_each_code = quote! {
        #(
             let option_item_ref = & (#vars);
             match option_item_ref {
                None => {}
                Some(ref item_ref) => { #for_each_run_block }
             }
        )*
    };

    let out: proc_macro::TokenStream = for_each_code.into();
    out
}



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
