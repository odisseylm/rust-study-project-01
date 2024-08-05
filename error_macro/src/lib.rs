mod macro_util;
mod error_source;
mod struct_error;
mod bt;

use crate::struct_error::{ impl_struct_error, impl_struct_error_source };


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
