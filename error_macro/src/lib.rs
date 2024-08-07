mod macro_util;
mod error_source;
mod struct_error;
mod bt;
mod this_error_ext;
mod this_error_bt_src_ext;

use crate::struct_error::{impl_struct_error, impl_struct_error_source };
use crate::this_error_ext::{impl_this_error_bt_src, impl_this_error_ext};

#[proc_macro_derive(StructError, attributes(
    StaticStructErrorType,
    do_not_generate_display,
    do_not_generate_debug,
    struct_error_internal_type_path_mode,
))]
pub fn struct_error_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse(input)
        .expect("No/incorrect input for derive macro StructError");
    impl_struct_error(&ast)
}


#[proc_macro_derive(StructErrorSource,
    attributes(
        // top
        struct_error_type,
        from_error_kind,
        struct_error_internal_type_path_mode,
        do_not_generate_display,
        do_not_generate_std_error,
        // enum variant
        no_source_backtrace,
        no_std_error,
    )
)]
pub fn struct_error_source_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input)
        .expect("No/incorrect input source for derive macro StructErrorSource.");
    impl_struct_error_source(&ast)
}


#[proc_macro_derive(ThisErrorFromWithBacktrace,
    attributes(
        // top
        struct_error_internal_type_path_mode,
        // do_not_generate_display,
        // enum variant
        from_with_bt, from_bt,
        inherit_or_capture_bt, inherit_or_capture,
        no_source_backtrace,
        no_std_error,
    )
)]
pub fn this_error_ext_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input)
        .expect("No/incorrect input source for derive macro StructErrorSource.");
    impl_this_error_ext(&ast)
}


#[proc_macro_derive(ThisErrorBacktraceSource,
    attributes(
        // top
        struct_error_internal_type_path_mode,
        // enum variant
        no_source_backtrace,
        // no_std_error,
        //
        std_error,
        skip_bt_source, std_error_bt_source,
    )
)]
pub fn this_error_bt_src_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input)
        .expect("No/incorrect input source for derive macro StructErrorSource.");
    impl_this_error_bt_src(&ast)
}
