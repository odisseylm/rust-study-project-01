use quote::quote;
use crate::feature_cfg::MAX_TUPLE_LEN;
use crate::tuple_for_each::parse_for_each_params;
use crate::util::make_ident;


pub(crate) fn tuple_find_some_by_ref(params: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let params = parse_for_each_params(params);

    if let Some(tuple_len) = params.tuple_len {
        tuple_find_some_by_ref_n(params.item_var_name.as_str(), tuple_len, params.tuple, params.item_processing_code)
    } else {
        tuple_find_some_by_ref_via_tuple_accessor(params.item_var_name.as_str(), params.tuple, params.item_processing_code)
    }
}


pub(crate) fn tuple_find_some_by_ref_n(
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
            quote! { (tuple_as_var_123456789.#i) }
        })
        .collect::<Vec<_>>();

    let for_each_code = quote! {
        {
            let tuple_as_var_123456789 = (#tuple);
            tuple_heter_iter::#assert_tuple_len_is_xxx( &tuple_as_var_123456789 );

            let mut comlex_result_123456789 = None;
            #(
                if comlex_result_123456789.is_none() {
                    let #var_ident = & (#vars);
                    let result = { #for_each_run_block };
                    comlex_result_123456789 = result;
                }
            )*

            comlex_result_123456789
        }
    };

    let out: proc_macro::TokenStream = for_each_code.into();
    out
}


pub(crate) fn tuple_find_some_by_ref_via_tuple_accessor(
    var_name: &str,
    tuple: proc_macro2::TokenStream,
    for_each_run_block: proc_macro2::TokenStream,
) -> proc_macro::TokenStream {

    let var_ident = make_ident(var_name.to_string());

    let vars = (0..MAX_TUPLE_LEN)
        .into_iter()
        .map(|i|{
            let method_ident = make_ident(format!("_{i}"));
            quote!{ tuple_as_var_123456789.#method_ident() }
        })
        .collect::<Vec<_>>();

    let for_each_code = quote! {
        {
            use tuple_heter_iter::TupleAccess;
            // It is need if 'tuple' expr is function (to avoid loosing temp object).

            let tuple_as_var_123456789 = (#tuple);
            let mut comlex_result_123456789 = None;

            #(
                if comlex_result_123456789.is_none() {
                    let option_item_ref = & (#vars);
                    match option_item_ref {
                        None => {}
                        Some(ref #var_ident) => {
                            let result = { #for_each_run_block };
                            comlex_result_123456789 = result;
                        }
                     }
                }
            )*

            comlex_result_123456789
        }
    };

    let out: proc_macro::TokenStream = for_each_code.into();
    out
}
