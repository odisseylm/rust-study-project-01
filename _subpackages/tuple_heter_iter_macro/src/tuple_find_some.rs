use quote::quote;
use crate::feature_cfg::MAX_TUPLE_LEN;
use crate::tuple_for_each::{ForEachInputParams, parse_for_each_params};
use crate::util::make_ident;


pub(crate) fn tuple_find_some_by_ref(params: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let params = parse_for_each_params(params);

    if let Some(ref _tuple_len) = params.tuple_len {
        tuple_find_some_by_ref_n(params)
    } else {
        tuple_find_some_by_ref_via_tuple_accessor(params)
    }
}


pub(crate) fn tuple_find_some_by_ref_n (params: ForEachInputParams) -> proc_macro::TokenStream {

    let var_ident = params.item_var_ident;
    let tuple = params.tuple;
    let tuple_len: usize = params.tuple_len.expect("tuple_for_each_by_ref_n requires 'tuple_len'.");
    let item_processing_code = params.item_processing_code;

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
                    let result = { #item_processing_code };
                    comlex_result_123456789 = result;
                }
            )*

            comlex_result_123456789
        }
    };

    let out: proc_macro::TokenStream = for_each_code.into();
    out
}


pub(crate) fn tuple_find_some_by_ref_via_tuple_accessor (params: ForEachInputParams)
    -> proc_macro::TokenStream {

    let var_ident = params.item_var_ident;
    let tuple = params.tuple;
    let item_processing_code = params.item_processing_code;
    let max_tuple_len = MAX_TUPLE_LEN;

    let vars = (0..max_tuple_len)
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
                            let result = { #item_processing_code };
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
