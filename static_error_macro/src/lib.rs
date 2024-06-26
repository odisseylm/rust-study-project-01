mod macro_util;
mod error_source;

mod compile_log;

// RustRover does not pick it up (however cargo does)
// #[macro_use]
// mod compile_log_macros;
include!("./compile_log_macros.rs");


use std::collections::{ HashMap, HashSet };
use if_chain::if_chain;
use itertools::*;

use quote::quote;
use crate::macro_util::*;
use crate::error_source::*;


fn bt_root_path_segment(path_mode: InternalTypePathMode) -> proc_macro2::TokenStream {
    match path_mode {
        InternalTypePathMode::InternalCratePath => quote! { crate },
        InternalTypePathMode::ExternalCratePath => quote! { :: project01 },
    }
}

fn bt_type(path_mode: InternalTypePathMode, type_name: &str) -> proc_macro2::TokenStream {
    let root = bt_root_path_segment(path_mode);
    let type_name_ident: syn::Ident = syn::parse_str(type_name).expect(&format!("Error of converting [{}] to Ident.", type_name));
    quote! { #root ::util::backtrace:: #type_name_ident }
}

/*

// The same but with direct using syn::Type.
fn bt_type(path_mode: InternalTypePathMode, type_name: &str) -> syn::Type {
    let use_path_expr_str = &format!("{}::util::backtrace::{}", bt_root_path_segment(path_mode), type_name);
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
fn use_bt_types_expr(path_mode: InternalTypePathMode) -> proc_macro2::TokenStream {
    let root = bt_root_path_segment(path_mode);
    quote! { use #root ::util::backtrace::{ BacktraceInfo, NewBacktracePolicy, InheritBacktracePolicy, BacktraceCopyProvider, BacktraceBorrowedProvider } ; }
}


#[proc_macro_derive(MyStaticStructError, attributes(StaticStructErrorType, do_not_generate_display, do_not_generate_debug, static_struct_error_internal_type_path_mode))]
pub fn my_static_struct_error_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast: syn::DeriveInput = syn::parse(input)
        .expect("No input for derive macro MyStaticStructError");

    impl_my_static_struct_error(&ast)
}

fn impl_my_static_struct_error(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let error_type_name = &ast.ident;

    let do_not_generate_display = find_attr(&ast.attrs, "do_not_generate_display").is_some();
    let do_not_generate_debug = find_attr(&ast.attrs, "do_not_generate_debug").is_some();

    let int_type_path_mode = get_internal_type_path_mode(ast);
    let root_type_path = bt_root_path_segment(int_type_path_mode);

    let use_bt_types = use_bt_types_expr(int_type_path_mode);
    #[allow(non_snake_case)]
    let BacktraceInfo = bt_type(int_type_path_mode, "BacktraceInfo");
    #[allow(non_snake_case)]
    let NewBacktracePolicy = bt_type(int_type_path_mode, "NewBacktracePolicy");
    #[allow(non_snake_case)]
    let BacktraceCopyProvider = bt_type(int_type_path_mode, "BacktraceCopyProvider");


    /*
    let source_field_exists: bool = if let syn::Data::Struct(ref data) = ast.data {
        if let syn::Fields::Named(ref fields) = data.fields {
            fields.named.iter().any(|f|
                f.ident
                .as_ref()
                .map(|ident_name| ident_name == "source")
                .unwrap_or(false)
            )
        }
        else { false }
    } else { false };
    */

    let source_field_exists: bool = if_chain! {
        if let syn::Data::Struct(ref data) = ast.data;
        if let syn::Fields::Named(ref fields) = data.fields;
        then {
            fields.named.iter().any(|f|
                f.ident
                .as_ref()
                .map(|ident_name| ident_name == "source")
                .unwrap_or(false)
            )
        } else { false }
    };

    let err_impl_without_source = quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl #error_type_name {

            pub fn new(kind: ErrorKind) -> Self {
                // Really is not needed there, but let's keep it just to test usage of generated 'use'.
                #use_bt_types

                Self { kind, backtrace: #BacktraceInfo ::new() }
            }
            pub fn with_backtrace(kind: ErrorKind, backtrace_policy: #NewBacktracePolicy) -> Self {
                Self { kind, backtrace: #BacktraceInfo ::new_by_policy(backtrace_policy) }
            }
        }

    };

    let err_impl_with_source = quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl #error_type_name {
            pub fn new(kind: ErrorKind) -> Self {
                // Really is not needed there, but let's keep it just to test usage of generated 'use'.
                #use_bt_types

                Self { kind, backtrace: #BacktraceInfo ::new(), source: ErrorSource::NoSource }
            }
            pub fn with_backtrace(kind: ErrorKind, backtrace_policy: #NewBacktracePolicy) -> Self {
                Self { kind, backtrace: #BacktraceInfo ::new_by_policy(backtrace_policy), source: ErrorSource::NoSource }
            }
            pub fn with_source(kind: ErrorKind, source: ErrorSource) -> Self {
                Self { kind, backtrace: #BacktraceInfo ::inherit_from(&source), source }
            }
            pub fn with_from<ES: Into<ErrorSource>>(kind: ErrorKind, source: ES) -> Self {
                let src = source.into();
                Self { kind, backtrace: #BacktraceInfo ::inherit_from(&src), source: src }
            }
        }

    };

    let err_backtrace_provider_impl = quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl #BacktraceCopyProvider for #error_type_name {
            #[warn(unused_must_use)]
            fn provide_backtrace(&self) -> #BacktraceInfo { self.backtrace.clone() }
        }
    };

    let err_display_impl = if do_not_generate_display { quote!() } else { quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl core::fmt::Display for #error_type_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, concat!(stringify!(#error_type_name), " {{ {} }}"), self.kind)
            }
        }
    } };

    let err_debug_impl_with_source = quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl core::fmt::Debug for #error_type_name {

            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                #root_type_path ::util::error::__private::error_debug_fmt_impl(
                    f, self, stringify!(#error_type_name), |er|&er.kind, |er|&er.source, |er|&er.backtrace)
            }

            // fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            //     if self.backtrace.is_captured() {
            //         let src_contains_captured_backtrace: bool = #BacktraceCopyProvider::contains_self_or_child_captured_backtrace(&self.source);
            //         if src_contains_captured_backtrace {
            //             write!(f, concat!(stringify!(#error_type_name), " {{ kind: {:?}, source: {:?} }}"), self.kind, self.source)
            //         } else {
            //             write!(f, concat!(stringify!(#error_type_name), " {{ kind: {:?}, source: {:?}, backtrace: {} }}"), self.kind, self.source, self.backtrace)
            //         }
            //     } else {
            //         write!(f, concat!(stringify!(#error_type_name), " {{ kind: {:?}, source: {:?} }}"), self.kind, self.source)
            //     }
            // }
        }
    };

    let err_debug_impl_without_source = quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl core::fmt::Debug for #error_type_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, concat!(stringify!(#error_type_name), " {{ kind: {:?}, backtrace: {} }}"), self.kind, self.backtrace)
            }
        }
    };

    let err_impl: proc_macro2::TokenStream = if source_field_exists { err_impl_with_source } else { err_impl_without_source };
    let err_debug_impl: proc_macro2::TokenStream = if do_not_generate_debug { quote!() } else {
        if source_field_exists { err_debug_impl_with_source } else { err_debug_impl_without_source }
    };

    // let mut all = proc_macro::TokenStream::new();
    // all.add_pm2_ts(err_impl);
    // all.add_pm2_ts(err_backtrace_provider_impl);
    // if !do_not_generate_display { all.add_pm2_ts(err_display_impl) };
    // if !do_not_generate_debug { all.add_pm2_ts(err_debug_impl) };
    // all

    // as separate var to avoid warn/error in RustRover
    let out = quote! {
        #err_impl
        #err_backtrace_provider_impl
        #err_display_impl
        #err_debug_impl
    };

    out.into()
}



#[proc_macro_derive(MyStaticStructErrorSource,
    attributes(
        struct_error_type,
        from_error_kind,
        no_source_backtrace,
        do_not_generate_display,
        do_not_generate_std_error,
        static_struct_error_internal_type_path_mode,
    )
)]
pub fn my_static_struct_error_source_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input)
        .expect("No input source for derive macro MyStaticStructErrorSource.");
    impl_my_static_struct_error_source(&ast)
}

fn impl_my_static_struct_error_source(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let _name = ast.ident.to_string();

    let do_not_generate_debug = find_attr(&ast.attrs, "do_not_generate_debug").is_some();
    let do_not_generate_display = find_attr(&ast.attrs, "do_not_generate_display").is_some();
    let do_not_generate_std_error = find_attr(&ast.attrs, "do_not_generate_std_error").is_some();

    let int_type_path_mode = get_internal_type_path_mode(ast);

    // let use_bt_types = use_bt_types_expr(internal_type_path_mode);
    #[allow(non_snake_case)]
    let BacktraceInfo = bt_type(int_type_path_mode, "BacktraceInfo");
    #[allow(non_snake_case)]
    let BacktraceCopyProvider = bt_type(int_type_path_mode, "BacktraceCopyProvider");

    let struct_error_type_attr: Option<&syn::Attribute> = find_attr(&ast.attrs, "struct_error_type");
    let struct_error_type: Option<String> = struct_error_type_attr
        .and_then(|attr| attr_list_as_string(attr));
    let struct_error_type: String = struct_error_type
        .expect("struct_error_type should have format: struct_error_type(MyErrorStructName)");

    let error_source_enum = get_error_source_enum_variants(ast);
    let enum_variants: Vec<&ErrorSourceEnumVariant> = error_source_enum
        .variants.iter()
        .collect::<Vec<_>>();

    let grouped_err_enum_variants_by_arg_type: HashMap<String, Vec<&ErrorSourceEnumVariant>> = enum_variants.iter()
        .filter_map(|&vr| vr.first_arg_type.map(|first_arg_type| (type_to_string(first_arg_type), vr) ))
        .into_group_map();

    let duplicated_err_enum_src_types: HashSet<String> = grouped_err_enum_variants_by_arg_type.iter()
        .filter(|(_, enums_vec)| enums_vec.len() > 1)
        .map(|src_type_and_vars| src_type_and_vars.0.to_string())
        .collect();

    let err_src_bt_provider_impl_match_branches: Vec<proc_macro2::TokenStream> = enum_variants.iter().map(|vr|{
        let var_name = vr.name;
        let no_source_backtrace = find_enum_variant_attr(vr.variant, "no_source_backtrace").is_some();
        let is_arg_present = vr.first_arg_type.is_some();

        if !is_arg_present {
            quote!(  ErrorSource:: #var_name  => { #BacktraceInfo ::empty() }     )
        } else if no_source_backtrace {
            quote!(  ErrorSource:: #var_name (_)  => { #BacktraceInfo ::empty() }     )
        } else {
            quote!(  ErrorSource:: #var_name (ref src)  => { src.provide_backtrace() }  )
        }
    }).collect::<Vec<_>>();

    let err_src_bt_provider_impl_match_branches2: Vec<proc_macro2::TokenStream> = enum_variants.iter().map(|vr|{
        let var_name = vr.name;
        let no_source_backtrace = find_enum_variant_attr(vr.variant, "no_source_backtrace").is_some();
        let is_arg_present = vr.first_arg_type.is_some();

        if !is_arg_present {
            quote!(  ErrorSource:: #var_name  => { false }     )
        } else if no_source_backtrace {
            quote!(  ErrorSource:: #var_name (_)  => { false }     )
        } else {
            quote!(  ErrorSource:: #var_name (ref src)  => { src.contains_self_or_child_captured_backtrace() }  )
        }
    }).collect::<Vec<_>>();


    let types_to_wrap_with_enum_vr_name: Vec<&str> = vec!("char",
                                                          "i8", "i16", "i32", "i64", "i128",
                                                          "u8", "u16", "u32", "u64", "u128",
                                                          "usize", "isize",
                                                          "f32", "f64",
                                                          "String", "&str", "& 'static str",
    );

    let err_src_debug_impl_match_branches: Vec<proc_macro2::TokenStream> = enum_variants.iter().map(|vr|{
        let var_name = vr.name;
        let is_arg_present = vr.first_arg_type.is_some();

        let to_wrap_with_enum_name: bool = vr.first_arg_type
            .map(|arg_type| types_to_wrap_with_enum_vr_name.contains(&type_to_string_without_spaces(arg_type).as_str()))
            .unwrap_or(false);

        if is_arg_present && to_wrap_with_enum_name {
            quote!(  #var_name(ref src)  => { write!(f, concat!(stringify!(#var_name),"({:?})"), src) }  )
        } else if is_arg_present {
            quote!(  #var_name(ref src)  => { write!(f, "{:?}", src) }  )
        } else {
            quote!(  #var_name  => { write!(f, stringify!(#var_name)) }  )
        }
    }).collect::<Vec<_>>();

    let err_src_impl = quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl #BacktraceCopyProvider for ErrorSource {
            fn provide_backtrace(&self) -> #BacktraceInfo {
                use #BacktraceInfo;
                match self {
                    #(#err_src_bt_provider_impl_match_branches)*
                    // ErrorSource::NoSource => { BacktraceInfo::empty() }
                    // ErrorSource::ParseBigDecimalError(_)  => { BacktraceInfo::empty()  }
                    // ErrorSource::CurrencyFormatError(ref src) => { src.provide_backtrace() }
                }
            }
            fn contains_self_or_child_captured_backtrace(&self) -> bool {
                use #BacktraceInfo ;
                match self {
                    #(#err_src_bt_provider_impl_match_branches2)*
                    // ErrorSource::NoSource => { false }
                    // ErrorSource::ParseBigDecimalError(_)  => { false  }
                    // ErrorSource::CurrencyFormatError(ref src) => { src.contains_self_or_child_captured_backtrace() }
                }
            }
        }
    };

    let debug_err_src_impl = if do_not_generate_debug { quote!() } else {
        quote! {

            #[allow(unused_imports)]
            #[allow(unused_qualifications)]
            impl core::fmt::Debug for ErrorSource {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    use ErrorSource::*;
                    match self {
                        #(#err_src_debug_impl_match_branches)*
                        // NoSource => { write!(f, "No source") }
                        // ...
                    }
                }
            }

        }
    };

    let from_impl: Vec<proc_macro2::TokenStream> = enum_variants.iter().map(|vr|{
        let variant_enum_name: &proc_macro2::Ident = vr.name;

        let from_error_kind_attr = find_enum_variant_attr(vr.variant, "from_error_kind");
        if from_error_kind_attr.is_none() {
            return quote!()
        }

        let from_error_kind_attr = from_error_kind_attr
            .expect(&format!("from_error_kind attribute is expected for {}", variant_enum_name));

        let error_kind = attr_list_as_string(from_error_kind_attr)
            // old approach
            // .unwrap_or_else(|| panic!("from_error_kind attribute value is expected for {}", variant_enum_name));
            //
            // since September 2022 we can use such better syntax
            .expect(&format!("from_error_kind attribute value is expected for {}", variant_enum_name));

        let from_err_type_name = vr.first_arg_type
            .expect(&format!("first argument as type is expected for {}", variant_enum_name));

        let err_struct_name: syn::Type = syn::parse_str(struct_error_type.as_str())
            .expect(&format!("{:?} has incorrect syntax for type.", struct_error_type));

        let from_err_type_name = from_err_type_name;
        let err_struct_kind_name: syn::Ident = syn::parse_str(error_kind.as_str())
            .expect(&format!("error kind {} should be identifier.", error_kind));

        let from_err_type_name_string = type_to_string(from_err_type_name);
        let all_enum_vars_for_this_from_err_type: &Vec<&ErrorSourceEnumVariant<'_>> = grouped_err_enum_variants_by_arg_type
            .get(&from_err_type_name_string)
            .expect(&format!("Internal error: Enum variants for type {} are not found.", from_err_type_name_string));

        if all_enum_vars_for_this_from_err_type.len() > 1 {
            let all_other_enums = all_enum_vars_for_this_from_err_type.iter()
                .filter(|enum_vr| enum_vr.name != variant_enum_name )
                .map(|enum_var| enum_var.name.to_string() )
                .collect::<Vec<String>>();
            let all_other_enums_as_str = all_other_enums.join(", ");

            panic!("Enum [{}] uses arg/source type [{}] which is also used in other enums [{}]. \
                    It is impossible to implement From/Into for duplicates.",
                   variant_enum_name, from_err_type_name_string, all_other_enums_as_str);
        }

        quote! {
            // impl From<CurrencyFormatError> for ParseAmountError {
            //     fn from(error: CurrencyFormatError) -> Self { ParseAmountError::with_from(ErrorKind::IncorrectCurrency, error) }
            // }

            #[allow(unused_imports)]
            #[allow(unused_qualifications)]
            impl From< #from_err_type_name > for #err_struct_name {
                fn from(error: #from_err_type_name ) -> Self { #err_struct_name::with_from(ErrorKind:: #err_struct_kind_name, error) }
            }
        }
    }).collect::<Vec<_>>();

    let into_impl: Vec<proc_macro2::TokenStream> = enum_variants.iter().filter_map(|ref el| {
        let var_name: &syn::Ident = el.name;

        if el.first_arg_type.is_none() {
            return None;
        }

        let var_arg_type: &syn::Type = el.first_arg_type
            .expect(&format!("first_arg_type is expected for {}.", var_name));

        let arg_type_as_string = type_to_string_without_spaces(var_arg_type);
        if duplicated_err_enum_src_types.contains(&arg_type_as_string) {
            compile_log_info!("'Into' is not implemented for {}.{} because there are others enum variants \
            in [{}] with the same src/arg type [{}].",
                error_source_enum.name, var_name, error_source_enum.name, arg_type_as_string);
            return None;
        }

        Some( quote! {
            // impl Into<ErrorSource> for CurrencyFormatError {
            //     fn into(self) -> ErrorSource { ErrorSource::CurrencyFormatError22(self) }
            // }

            #[allow(unused_imports)]
            #[allow(unused_qualifications)]
            impl Into<ErrorSource> for #var_arg_type {
                fn into(self) -> ErrorSource { ErrorSource:: #var_name (self) }
            }
        })
    }).collect::<Vec<_>>();

    let display_err_src_impl = if do_not_generate_display { quote!() }
    else {
        let display_err_src_items_impl: Vec<proc_macro2::TokenStream> = enum_variants.iter().map(|ref el| {
            let var_name: &syn::Ident = el.name;
            let is_src_arg_present: bool = el.first_arg_type.is_some();
            let to_wrap_with_enum_name: bool = el.first_arg_type
                .map(|arg_type| types_to_wrap_with_enum_vr_name.contains(&type_to_string_without_spaces(arg_type).as_str()))
                .unwrap_or(false);

            if is_src_arg_present && to_wrap_with_enum_name {
                quote! { ErrorSource:: #var_name (ref src)  => { write!(f, concat!(stringify!(#var_name), "({})"), src) } }
            } else if is_src_arg_present {
                quote! { ErrorSource:: #var_name (ref src)  => { write!(f, "{}", src) } }
            } else {
                quote! { ErrorSource:: #var_name  => { write!(f, stringify!(#var_name)) } }
            }
        }).collect::<Vec<_>>();

        quote! {
            #[allow(unused_imports)]
            #[allow(unused_qualifications)]
            impl core::fmt::Display for ErrorSource {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    match self {
                        #(#display_err_src_items_impl)*
                        // ErrorSource::NoSource                => { write!(f, "NoSource") }
                        // // ErrorSource::CurrencyFormatError(_)  => { write!(f, "CurrencyFormatError")  }
                        // // ErrorSource::ParseBigDecimalError(_) => { write!(f, "ParseBigDecimalError") }
                        // ErrorSource::CurrencyFormatError(src)  => { write!(f, "{}", src) }
                        // ErrorSource::ParseBigDecimalError(src) => { write!(f, "{}", src) }
                    }
                }
            }

        }
    };

    let types_without_std_err: Vec<&str> = vec!("char",
                                     "i8", "i16", "i32", "i64", "i128",
                                     "u8", "u16", "u32", "u64", "u128",
                                     "usize", "isize",
                                     "f32", "f64",
                                     "String", "&str", "& 'static str",
    );

    let std_error_err_src_impl = if do_not_generate_std_error { quote!() }
    else {
        let fn_source_items_impl: Vec<proc_macro2::TokenStream> = enum_variants.iter().map(|ref el| {
            let var_name: &syn::Ident = el.name;
            let is_src_arg_present: bool = el.first_arg_type.is_some();
            let arg_str_type: Option<String> = el.first_arg_type
                .map(|arg_type| type_to_string_without_spaces(arg_type));
            let no_std_err_for_arg = el.first_arg_type
                .map(|arg_type| type_to_string(arg_type))
                .map(|arg_type_as_str| types_without_std_err.contains(&arg_type_as_str.as_str()))
                .unwrap_or(false);

            if !is_src_arg_present {
                quote! { ErrorSource:: #var_name => { None }  }
            } else if no_std_err_for_arg {
                quote! { ErrorSource:: #var_name (_) => { None } }
            } else if arg_str_type.is_eq_to_str("anyhow::Error") {
                quote! { ErrorSource:: #var_name (ref src) => { Some(src.as_ref()) } }
            } else if arg_str_type.is_eq_to_str("Box<dyn std::error::Error>")
                   || arg_str_type.is_eq_to_str("Box<dyn error::Error>")
                   || arg_str_type.is_eq_to_str("Box<dyn Error>") {
                quote! { ErrorSource:: #var_name (ref src) => { Some(src.as_ref()) } }
            } else {
                // T O D O: could we use some universal way with something like TryAsRef
                // quote! { ErrorSource:: #var_name (ref src) => { Some(src.as_ref()) } }
                quote! { ErrorSource:: #var_name (ref src) => { Some(src) } }
            }
        }).collect::<Vec<_>>();

        let fn_description_items_impl: Vec<proc_macro2::TokenStream> = enum_variants.iter().map(|ref el| {
            let var_name: &syn::Ident = el.name;
            let is_src_arg_present: bool = el.first_arg_type.is_some();
            let no_std_err_for_arg = el.first_arg_type
                .map(|arg_type| type_to_string(arg_type))
                .map(|arg_type_as_str| types_without_std_err.contains(&arg_type_as_str.as_str()))
                .unwrap_or(false);

            if is_src_arg_present {
                if no_std_err_for_arg {
                    quote! { ErrorSource:: #var_name (_)  => { "" } }
                } else {
                    quote! { ErrorSource:: #var_name (ref src)  => { src.description() } }
                }
            } else {
                quote! { ErrorSource:: #var_name => { "" } }
            }
        }).collect::<Vec<_>>();

        quote! {
            #[allow(unused_imports)]
            #[allow(unused_qualifications)]
            impl std::error::Error for ErrorSource {
                fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                    match self {
                        #(#fn_source_items_impl)*
                        // ErrorSource::NoSource => { None }
                        // ErrorSource::CurrencyFormatError(ref src)  => { Some(src) }
                        // ErrorSource::ParseBigDecimalError(ref src) => { Some(src) }
                        _ => { None }
                    }
                }

                #[allow(deprecated)]
                fn description(&self) -> &str {
                    match self {
                        #(#fn_description_items_impl)*
                        // ErrorSource::NoSource => { "" }
                        // ErrorSource::CurrencyFormatError(src)  => { src.description() }
                        // ErrorSource::ParseBigDecimalError(src) => { src.description() }
                    }
                }

                // fn provide<'a>(&'a self, request: &mut Request<'a>) { ... }
            }

        }
    };

    let err_impl_ts: proc_macro2::TokenStream = err_src_impl;

    // let mut all = proc_macro::TokenStream::new();
    // all.add_ts(err_impl_ts);
    // all.add_pm2_tss(into_impl);
    // all.add_pm2_tss(from_impl);
    // all.add_pm2_ts(debug_err_src_impl);
    // all.add_pm2_ts(display_err_src_impl);
    // all.add_pm2_ts(std_error_err_src_impl);
    // all

    // as separate var to avoid warn/error in RustRover
    let out = quote! {
        #err_impl_ts
        #(#into_impl)*
        #(#from_impl)*
        #debug_err_src_impl
        #display_err_src_impl
        #std_error_err_src_impl
    };

    out.into()
}


#[proc_macro]
pub fn for_each_by_ref(params: proc_macro::TokenStream) -> proc_macro::TokenStream {

    use proc_macro2::TokenTree;
    let params2: proc_macro2::TokenStream = params.into();

    let mut params_vec = Vec::<proc_macro2::TokenStream>::new();
    let mut current_param_as_stream = proc_macro2::TokenStream::new();

    use itertools::Itertools;

    for (pos, tt) in params2.into_iter().with_position() {

        let mut end_of_func_param = false;
        use quote::TokenStreamExt;

        if let TokenTree::Punct(ref punct) = tt {
            if punct.as_char() == ',' {
                end_of_func_param = true;
            }
            else {
                current_param_as_stream.append(tt);
            }
        } else {
            current_param_as_stream.append(tt);
        }

        if let Position::Last | Position::Only = pos {
            end_of_func_param = true;
        }

        if end_of_func_param {
            params_vec.push(current_param_as_stream);
            current_param_as_stream = proc_macro2::TokenStream::new();
        }
    }

    assert!(params_vec.len() >= 2, "Expected at least one param and run block.");

    let for_each_run_block = params_vec.pop()
        .expect("Expected at least one param and run block.");

    let for_each_code = quote! {
        #(
             let item_ref = & (#params_vec);
             #for_each_run_block
        )*
    };

    let out: proc_macro::TokenStream = for_each_code.into();
    out
}



// -------------------------------------------------------------------------------------------------
//                                        Private tests
// -------------------------------------------------------------------------------------------------


// Tests for private methods/behavior
// Other test are located in ${project}/tests/currency_test.rs
//
#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_of_error_source() {
    }
}
