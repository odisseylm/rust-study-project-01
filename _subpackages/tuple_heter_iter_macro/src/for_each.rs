use quote::quote;
use crate::util::{ make_ident, split_params };


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
