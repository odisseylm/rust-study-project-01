use quote::quote;
use crate::macro_util::{InternalTypePathMode, StringOp, type_to_string};
//--------------------------------------------------------------------------------------------------


pub const MY_BACKTRACE_AND_ERR_CRATE: &str = "mvv_common";


pub(crate) fn bt_type(path_mode: InternalTypePathMode, type_name: &str) -> proc_macro2::TokenStream {
    let root = bt_root_path_segment(path_mode);
    let type_name_ident: syn::Ident = syn::parse_str(type_name).expect(&format!("Error of converting [{}] to Ident.", type_name));
    quote! { #root ::backtrace:: #type_name_ident }
}


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


pub(crate) enum BacktraceTypeKind {
    BacktraceCell,
    StdBacktrace,
}

pub(crate) fn get_bt_type_kind(t: &syn::Type) -> Option<BacktraceTypeKind> {
    let bt_type_str = type_to_string(t);
    let bt_type_str = bt_type_str.as_str();

    if bt_type_str.is_eq_to_one_of_str(["BacktraceCell"])
        || bt_type_str.ends_with_one_of(["::BacktraceCell"]) {
        Some(BacktraceTypeKind::BacktraceCell)
    } else if bt_type_str.is_eq_to_one_of_str([
        "Backtrace", "std::backtrace::Backtrace", "StdBacktrace",
        "StdBacktraceAlias", "BacktraceAlias"]) {
        Some(BacktraceTypeKind::StdBacktrace)
    } else {
        None
    }
}

pub(crate) fn is_bt_type(t: &syn::Type) -> bool {
    get_bt_type_kind(t).is_some()
}


