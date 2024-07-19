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



/// Usage:
/// ```no_build
///   let r: axum::Route<...> = axum::Route::new();
///   let r = route_from_open_api!(r, call_rest_get_client_account::<AccountS, String>);
/// ```
/// You can add faked/unused '&' to suppress RustRover warning 'Value used after being moved'.
/// It will be ignored.
/// ``` let r = route_from_open_api!(&r, call_rest_get_client_account::<AccountS, String>); ```
#[proc_macro]
pub fn route_from_open_api(params: proc_macro::TokenStream) -> proc_macro::TokenStream {

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
            #route_var.route_from_open_api (
                & #utoipa_path_obj,
                #route_method,
            )
        }
    };
    q.into()
}


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

fn remove_leading_ref(param: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
    let as_str = param.to_string();
    if as_str.starts_with('&') {
        param.into_iter().skip(1).collect()
    } else {
        param
    }
}