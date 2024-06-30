use quote::quote;
use crate::feature_cfg::MAX_TUPLE_LEN;
use crate::util::{as_uint_literal, make_ident, split_params};


pub(crate) fn for_each_in_tuple_by_ref(params: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let params_vec = split_params(params);

    macro_rules! input_err_msg { () => { "Input should be {{ var, tuple_expr, tuple_len, {{ code }} }}." }; }

    assert_eq!(params_vec.len(), 4, input_err_msg!());

    let var_name = params_vec.get(0).expect(input_err_msg!())
        .to_string();
    let var_name = var_name
        .strip_prefix("$")
        .map(|s|s.to_string())
        // .expect("Var name should start from $").to_string();
        .unwrap_or(var_name);
    let var_ident = make_ident(var_name);

    let tuple = params_vec.get(1)
        .expect(input_err_msg!());
    let tuple_len_ts = params_vec.get(2)
        .expect(input_err_msg!());

    let count: Option<usize> = as_uint_literal(Some(tuple_len_ts.clone()));
    let count = count.expect("Expects tuple size as usize literal");

    let for_each_run_block = params_vec.get(3)
        .expect(input_err_msg!());

    let vars = (0..count)
        .into_iter()
        .map(|i|{
            let i = syn::Index::from(i);
            quote! { ((#tuple).#i) }
        })
        .collect::<Vec<_>>();

    let for_each_code = quote! {
        #(
             let #var_ident = & (#vars);
             #for_each_run_block
        )*
    };

    let out: proc_macro::TokenStream = for_each_code.into();
    out
}



// #[proc_macro]
pub(crate) fn for_each_in_tuple_by_ref_2(params: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let params_vec = split_params(params);

    // const input_err_msg = "Input should be { tuple_expr, { code } }.";
    macro_rules! input_err_msg { () => { "Input should be {{ var, tuple_expr, {{ code }} }}." }; }

    assert_eq!(params_vec.len(), 3, input_err_msg!());

    let var_name = params_vec.get(0).expect(input_err_msg!())
        .to_string();
    let var_name = var_name
        .strip_prefix("$")
        .map(|s|s.to_string())
        // .expect("Var name should start from $").to_string();
        .unwrap_or(var_name);
    let var_ident = make_ident(var_name);

    let tuple = params_vec.get(1).expect(input_err_msg!());
    let for_each_run_block = params_vec.get(2).expect(input_err_msg!());

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
                Some(ref #var_ident) => { #for_each_run_block }
             }
        )*
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
