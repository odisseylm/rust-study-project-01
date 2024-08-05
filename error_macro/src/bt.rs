use quote::quote;
use crate::macro_util::InternalTypePathMode;
//--------------------------------------------------------------------------------------------------



pub(crate) fn bt_type(path_mode: InternalTypePathMode, type_name: &str) -> proc_macro2::TokenStream {
    let root = bt_root_path_segment(path_mode);
    let type_name_ident: syn::Ident = syn::parse_str(type_name).expect(&format!("Error of converting [{}] to Ident.", type_name));
    quote! { #root ::backtrace:: #type_name_ident }
}

/*

// The same but with direct using syn::Type.
pub(crate) fn bt_type(path_mode: InternalTypePathMode, type_name: &str) -> syn::Type {
    let use_path_expr_str = &format!("{}::backtrace::{}", bt_root_path_segment(path_mode), type_name);
    let as_expr: syn::Type = syn::parse_str(use_path_expr_str)
        .expect(&format!("Internal error: invalid 'type' expr [{}].", use_path_expr_str));
    as_expr
}

fn use_bt_types_expr(path_mode: InternalTypePathMode) -> Vec<proc_macro2::TokenStream> { // Vec<proc_macro2::TokenStream> {
    vec!(
        "BacktraceInfo",
        "NewBacktracePolicy", "InheritBacktracePolicy",
        "BacktraceCopyProvider", "BacktraceBorrowedProvider",
    ).iter()
        .map(|bt_type_name| bt_type(path_mode, bt_type_name))
        .map(|bt_type| quote!( use #bt_type; ))
        .collect_vec()
}
*/

pub(crate) fn bt_root_path_segment(path_mode: InternalTypePathMode) -> proc_macro2::TokenStream {
    match path_mode {
        InternalTypePathMode::InternalCratePath => quote! { crate },
        InternalTypePathMode::ExternalCratePath => quote! { :: mvv_common },
    }
}

pub(crate) fn use_bt_types_expr(path_mode: InternalTypePathMode) -> proc_macro2::TokenStream {
    let root = bt_root_path_segment(path_mode);
    quote! { use #root ::backtrace::{ BacktraceCell, BacktraceSource } ; }
}
