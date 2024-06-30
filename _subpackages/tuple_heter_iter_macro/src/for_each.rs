use quote::quote;
use crate::feature_cfg::MAX_TUPLE_LEN;
use crate::util::{ as_uint_literal, make_ident, split_params };


pub(crate) fn for_each_in_tuple_by_ref (
    params: proc_macro::TokenStream,
) -> proc_macro::TokenStream {

    let mut params_vec = split_params(params);

    macro_rules! input_err_msg { () => { "Input should be {{ var, tuple_expr[, tuple_len], {{ code }} }}." }; }

    let param_count = params_vec.len();
    assert!(param_count == 3 || param_count == 4, input_err_msg!());

    let for_each_run_block = params_vec.pop()
        .expect(input_err_msg!());

    let tuple_len: Option<usize> = if param_count == 4 {
        let tuple_len_ts = params_vec.pop().expect(input_err_msg!());
        Some(as_uint_literal(Some(tuple_len_ts.clone()))
            .expect("Expects tuple size as usize literal"))
    } else { None };

    let tuple = params_vec.pop().expect(input_err_msg!());

    let var_name = params_vec.pop().expect(input_err_msg!())
        .to_string();
    let var_name = var_name
        .strip_prefix("$")
        .map(|s|s.to_string())
        // .expect("Var name should start from $").to_string();
        .unwrap_or(var_name);

    if let Some(tuple_len) = tuple_len {
        for_each_in_tuple_by_ref_n(var_name.as_str(), tuple_len, tuple, for_each_run_block)
    } else {
        for_each_in_tuple_by_ref_via_tuple_accessor(var_name.as_str(), tuple, for_each_run_block)
    }
}


pub(crate) fn for_each_in_tuple_by_ref_n (
    var_name: &str,
    tuple_len: usize,
    tuple: proc_macro2::TokenStream,
    for_each_run_block: proc_macro2::TokenStream,
) -> proc_macro::TokenStream {

    let var_ident = make_ident(var_name.to_string());
    let assert_tuple_len_is_xxx = make_ident(format!("assert_tuple_len_is_{tuple_len}"));

    let vars = (0..tuple_len)
        .into_iter()
        .map(|i|{
            let i = syn::Index::from(i);
            quote! { ((#tuple).#i) }
        })
        .collect::<Vec<_>>();

    let for_each_code = quote! {
        #(
            tuple_heter_iter::#assert_tuple_len_is_xxx( &(#tuple) );

             let #var_ident = & (#vars);
             #for_each_run_block
        )*
    };

    let out: proc_macro::TokenStream = for_each_code.into();
    out
}


pub(crate) fn for_each_in_tuple_by_ref_via_tuple_accessor(
    var_name: &str,
    tuple: proc_macro2::TokenStream,
    for_each_run_block: proc_macro2::TokenStream,
) -> proc_macro::TokenStream {

    let var_ident = make_ident(var_name.to_string());
    let max_tuple_len = MAX_TUPLE_LEN;

    let mut vars = Vec::<proc_macro2::TokenStream>::new();
    for i in 0..max_tuple_len {
        let method_ident = make_ident(format!("_{i}"));
        vars.push( quote!{ tuple_as_var_123456789.#method_ident() } );
    }

    let for_each_code = quote! {
        {
            use tuple_heter_iter::TupleAccess;
            // It is need if 'tuple' expr is function (to avoid loosing temp object).
            let tuple_as_var_123456789 = (#tuple);

            #(
                 let option_item_ref = & (#vars);
                 match option_item_ref {
                    None => {}
                    Some(ref #var_ident) => { #for_each_run_block }
                 }
            )*
        }
    };

    let out: proc_macro::TokenStream = for_each_code.into();
    out
}


// #[proc_macro]
pub(crate) fn for_each_by_ref(params: proc_macro::TokenStream) -> proc_macro::TokenStream {

    // if true { return proc_macro::TokenStream::new(); }

    macro_rules! input_err_msg { () => { "Input should be {{ loop var, items to iterate over, {{ code }} }}." }; }

    let mut params_vec = split_params(params);
    assert!(params_vec.len() >= 3, input_err_msg!());

    let var_name = params_vec.remove(0)
        //.expect(input_err_msg!())
        .to_string();
    let var_name = var_name
        .strip_prefix("$")
        .map(|s|s.to_string())
        // .expect("Var name should start from $").to_string();
        .unwrap_or(var_name);
    let var_ident = make_ident(var_name);

    let for_each_run_block = params_vec.pop()
        .expect(input_err_msg!());

    let for_each_code = quote! {
        #(
             let #var_ident = & (#params_vec);
             #for_each_run_block
        )*
    };

    let out: proc_macro::TokenStream = for_each_code.into();
    out
}
