use quote::quote;
use crate::feature_cfg::MAX_TUPLE_LEN;
use crate::util::{ as_uint_literal, make_ident, split_params };


pub(crate) fn tuple_for_each_by_ref(params: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let params = parse_for_each_params(params);

    if let Some(ref _tl) = params.tuple_len {
        tuple_for_each_by_ref_n(params)
    } else {
        tuple_for_each_by_ref_via_tuple_accessor(params)
    }
}

pub(crate) struct ForEachInputParams {
    // pub(crate) item_var_name: String,
    pub(crate) item_var_ident: proc_macro2::TokenStream,
    pub(crate) tuple: proc_macro2::TokenStream,
    pub(crate) tuple_len: Option<usize>,
    pub(crate) item_processing_code: proc_macro2::TokenStream,
}

pub(crate) fn parse_for_each_params(params: proc_macro::TokenStream) -> ForEachInputParams {

    let mut params_vec = split_params(params);

    macro_rules! input_err_msg { () => { "Input should be {{ var, tuple_expr[, tuple_len], {{ code }} }}." }; }

    let param_count = params_vec.len();
    assert!(param_count == 3 || param_count == 4, input_err_msg!());

    let item_processing_code = params_vec.pop()
        .expect(input_err_msg!());

    let tuple_len: Option<usize> = if param_count == 4 {
        let tuple_len_ts = params_vec.pop().expect(input_err_msg!());
        Some(as_uint_literal(Some(tuple_len_ts.clone()))
            .expect("Expects tuple size as usize literal"))
    } else { None };

    let tuple = params_vec.pop().expect(input_err_msg!());

    let var_name = params_vec.pop().expect(input_err_msg!())
        .to_string();
    let item_var_name = var_name
        .strip_prefix("$")
        .map(|s|s.to_string())
        // .expect("Var name should start from $").to_string();
        .unwrap_or(var_name);

    ForEachInputParams {
        item_var_ident: make_ident(item_var_name.to_string()),
        // item_var_name,
        tuple,
        tuple_len,
        item_processing_code,
    }
}


pub(crate) fn tuple_for_each_by_ref_n (
    params: ForEachInputParams,
) -> proc_macro::TokenStream {

    let var_ident = params.item_var_ident;
    let tuple = params.tuple;
    let tuple_len: usize = params.tuple_len.expect("tuple_for_each_by_ref_n requires 'tuple_len'.");
    let item_processing_code = params.item_processing_code;

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
            mvv_tuple_heter_iter::#assert_tuple_len_is_xxx( &(#tuple) );

             let #var_ident = & (#vars);
             #item_processing_code
        )*
    };

    let out: proc_macro::TokenStream = for_each_code.into();
    out
}


pub(crate) fn tuple_for_each_by_ref_via_tuple_accessor (
    params: ForEachInputParams,
) -> proc_macro::TokenStream {

    let var_ident = params.item_var_ident;
    let tuple = params.tuple;
    let item_processing_code = params.item_processing_code;
    let max_tuple_len = MAX_TUPLE_LEN;

    let mut vars = Vec::<proc_macro2::TokenStream>::new();
    for i in 0..max_tuple_len {
        let method_ident = make_ident(format!("_{i}"));
        vars.push( quote!{ tuple_as_var_123456789.#method_ident() } );
    }

    let for_each_code = quote! {
        {
            use mvv_tuple_heter_iter::TupleAccess;
            // It is need if 'tuple' expr is function (to avoid loosing temp object).
            let tuple_as_var_123456789 = (#tuple);

            #(
                 let option_item_ref = & (#vars);
                 match option_item_ref {
                    None => {}
                    Some(ref #var_ident) => { #item_processing_code }
                 }
            )*
        }
    };

    let out: proc_macro::TokenStream = for_each_code.into();
    out
}
