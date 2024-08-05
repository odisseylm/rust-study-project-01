use quote::quote;
use crate::macro_util::InternalTypePathMode;
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
