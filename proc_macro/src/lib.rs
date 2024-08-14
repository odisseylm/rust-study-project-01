use quote::quote;
use crate::macro_util::*;
use crate::utils::{ make_ident, split_params };
use StringOp;

// RustRover does not pick it up (however cargo does)
// #[macro_use]
// mod compile_log_macros;
// include!("compile_log_macros.rs");
//--------------------------------------------------------------------------------------------------

mod macro_util;
mod utils;
//--------------------------------------------------------------------------------------------------


#[proc_macro]
pub fn utoipa_path_obj(params: proc_macro::TokenStream) -> proc_macro::TokenStream {

    macro_rules! input_err_msg { () => { "Expects 1 params" }; }

    let mut params = split_params(params);
    assert_eq!(params.len(), 1, input_err_msg!());

    let route_method = params.pop().expect(input_err_msg!());
    let route_method_name: String = get_method_name(&route_method.to_string());
    let utoipa_path_obj = make_ident(format!("__path_{route_method_name}"));

    let q = quote! { #utoipa_path_obj };
    q.into()
}

// Actually similar macro_rules version is better since IDE properly process macro_rules macros
// without unexpected warnings and proper switching to methods and types.
//
/*
/// Usage:
/// ```no_build
///   let r: axum::Route<...> = axum::Route::new();
///   let r = axum_route_from_open_api!(r, call_rest_get_client_account::<AccountS, String>);
/// ```
/// You can add faked/unused '&' to suppress RustRover warning 'Value used after being moved'.
/// It will be ignored.
/// ``` let r = axum_route_from_open_api!(&r, call_rest_get_client_account::<AccountS, String>); ```
#[proc_macro]
pub fn axum_route_from_open_api(params: proc_macro::TokenStream) -> proc_macro::TokenStream {

    macro_rules! input_err_msg { () => { "Expects 2 params" }; }

    let mut params = split_params(params);
    assert_eq!(params.len(), 2, input_err_msg!());

    let route_method = params.pop().expect(input_err_msg!());
    let route_var = remove_leading_ref(
        params.pop().expect(input_err_msg!()));

    let route_method_name: String = get_method_name(&route_method.to_string());
    let utoipa_path_obj = make_ident(format!("__path_{route_method_name}"));

    let building_crate = std::env::var("CARGO_CRATE_NAME")
        .ok().unwrap_or(String::new());
    let root_path_prefix =
            if building_crate == "mvv_common" { quote! { crate } }
            else { quote! { mvv_common } };

    let q = quote! {
        {
            use #root_path_prefix ::utoipa::OpenApiRouterExt;
            #route_var.route_from_open_api_internal (
                & #utoipa_path_obj,
                #route_method,
            )
        }
    };
    q.into()
}

fn remove_leading_ref(param: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let as_str = param.to_string();
    if as_str.starts_with('&') {
        param.into_iter().skip(1).collect()
    } else {
        param
    }
}
*/

fn get_method_name(method_with_gen_params: &str) -> String {
    let route_method_name: String = method_with_gen_params.remove_space_chars();
    let method_name_end = route_method_name.find("::");
    let route_method_name: String = if let Some(method_name_end) = method_name_end {
        route_method_name[0..method_name_end].to_string()
    } else {
        route_method_name
    };
    route_method_name
}


/*
#[proc_macro_attribute]
pub fn integration_test(attrs: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {

    let fn_item: syn::ItemFn = syn::parse(item)
        .expect("[integration_test] macro expects function");
    let syn::ItemFn { attrs: item_attrs, vis, sig, block } = fn_item;
    let attrs: proc_macro2::TokenStream = attrs.into();

    let is_it_1 = std::env::var("INTEGRATION_TEST").is_ok();
    let is_it_2 = std::env::var("INTEGRATION_TESTS").is_ok();
    let is_exact = std::env::args_os().contains(&OsString::from("--exact"));

    let test_enabled = is_it_1 || is_it_2 || is_exact;

    let out = if test_enabled {
        quote! {
            #attrs
            #(#item_attrs)*
            #vis #sig #block
        }
    } else {
        let ignore_msg = quote! { "Integration tests are not enabled" };
        quote! {
            #attrs
            #(#item_attrs)*
            #[ignore = #ignore_msg ]
            #vis #sig #block
        }
    };

    let out: proc_macro2::TokenStream = out.into();
    out.into()
}
*/
