use std::collections::{HashMap, HashSet};
use if_chain::if_chain;
use itertools::Itertools;
use quote::quote;
use crate::{
    bt::{ bt_root_path_segment, bt_type, use_bt_types_expr },
    error_source::{
        ErrorSourceEnum, ErrorSourceEnumVariant,
        get_error_source_enum_variants, get_internal_type_path_mode,
    },
    macro_util::{
        attr_list_as_string, find_attr, find_enum_variant_attr,
        InternalTypePathMode, OptionStringOp,
        type_to_string, type_to_string_without_spaces,
    },
};
use crate::macro_util::keys_with_duplicates;
//--------------------------------------------------------------------------------------------------


// RustRover does not pick it up (however cargo does)
// #[macro_use]
// mod compile_log_macros;
include!("compile_log_macros.rs");



pub(crate) fn impl_struct_error(ast: &syn::DeriveInput) -> proc_macro::TokenStream {

    let GenerateStructErrorAttrs { do_not_generate_display, do_not_generate_debug } =
        GenerateStructErrorAttrs::new(ast);

    let source_field_exists = is_source_field_present(ast);

    let err_impl_without_source = generate_struct_error_without_source_field_impl_block(ast);
    let err_impl_with_source = generate_struct_error_with_source_field_impl_block(ast);
    let err_backtrace_provider_impl = generate_struct_error_backtrace_source_impl_block(ast);

    let err_display_impl = if do_not_generate_display { quote!() }
    else { generate_struct_error_display_impl_block(ast) };

    let err_debug_impl_with_source = generate_struct_error_with_source_debug_impl_block(ast);
    let err_debug_impl_without_source = generate_struct_error_without_source_debug_impl_block(ast);

    let err_impl: proc_macro2::TokenStream =
        if source_field_exists { err_impl_with_source }
        else { err_impl_without_source };

    let err_debug_impl: proc_macro2::TokenStream =
        if do_not_generate_debug { quote!() }
        else if source_field_exists { err_debug_impl_with_source }
        else { err_debug_impl_without_source };

    // as separate var to avoid warn/error in RustRover
    let out = quote! {
        #err_impl
        #err_backtrace_provider_impl
        #err_display_impl
        #err_debug_impl
    };

    out.into()
}


#[allow(non_snake_case)]
struct GenerateStructErrorAttrs {
    do_not_generate_display: bool,
    do_not_generate_debug: bool,
    // struct_error_internal_type_path_mode: InternalTypePathMode,
}
impl GenerateStructErrorAttrs {
    fn new(ast: &syn::DeriveInput) -> Self {
        Self {
            do_not_generate_display: find_attr( &ast.attrs, "do_not_generate_display").is_some(),
            do_not_generate_debug: find_attr( &ast.attrs, "do_not_generate_debug").is_some(),
            // struct_error_internal_type_path_mode: ,
        }
    }
}


#[allow(non_snake_case)]
struct GenerateStructErrorCfg {
    error_type_name: proc_macro2::Ident,
    #[allow(dead_code)]
    internal_type_path_mode: InternalTypePathMode,
    // expressions
    root_type_path: proc_macro2::TokenStream,
    use_bt_types: proc_macro2::TokenStream,
    BacktraceCell: proc_macro2::TokenStream,
    BacktraceSource: proc_macro2::TokenStream,
}
impl GenerateStructErrorCfg {
    fn new(ast: &syn::DeriveInput) -> Self {
        let internal_type_path_mode = get_internal_type_path_mode(ast);
        Self {
            error_type_name: ast.ident.clone(),
            internal_type_path_mode,
            // expressions
            root_type_path: bt_root_path_segment(internal_type_path_mode),
            use_bt_types: use_bt_types_expr(internal_type_path_mode),
            BacktraceCell: bt_type(internal_type_path_mode, "BacktraceCell"),
            BacktraceSource: bt_type(internal_type_path_mode, "BacktraceSource"),
        }
    }
}


struct StaticStructErrorSourceAttrs {
    do_not_generate_debug: bool,
    do_not_generate_display: bool,
    do_not_generate_std_error: bool,
    struct_error_type: String,
}
impl StaticStructErrorSourceAttrs {
    fn new(ast: &syn::DeriveInput) -> Self {

        let struct_error_type_attr: Option<&syn::Attribute> = find_attr(&ast.attrs, "struct_error_type");
        let struct_error_type: Option<String> = struct_error_type_attr
            .and_then(|attr| attr_list_as_string(attr));
        let struct_error_type: String = struct_error_type
            .expect("struct_error_type should have format: struct_error_type(MyErrorStructName)");

        Self {
            do_not_generate_debug: find_attr(&ast.attrs, "do_not_generate_debug").is_some(),
            do_not_generate_display: find_attr(&ast.attrs, "do_not_generate_display").is_some(),
            do_not_generate_std_error: find_attr(&ast.attrs, "do_not_generate_std_error").is_some(),
            struct_error_type,
        }
    }
}


fn generate_struct_error_without_source_field_impl_block(ast: &syn::DeriveInput)
    -> proc_macro2::TokenStream {

    #[allow(non_snake_case)]
    let GenerateStructErrorCfg { error_type_name, use_bt_types, BacktraceCell, .. } =
        GenerateStructErrorCfg::new(ast);

    let err_impl_without_source = quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl #error_type_name {

            #[inline]
            pub fn new(kind: ErrorKind) -> Self {
                Self::with_backtrace(kind)
            }
            pub fn with_backtrace(kind: ErrorKind) -> Self {
                Self { kind, backtrace: #BacktraceCell ::capture_backtrace() }
            }
            pub fn without_backtrace(kind: ErrorKind) -> Self {
                // Really is not needed there, but let's keep it just to test usage of generated 'use'.
                #use_bt_types

                Self { kind, backtrace: #BacktraceCell ::empty() }
            }
        }

    };

    err_impl_without_source
}


fn is_source_field_present(ast: &syn::DeriveInput) -> bool {
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

    source_field_exists
}

fn generate_struct_error_with_source_field_impl_block(ast: &syn::DeriveInput)
    -> proc_macro2::TokenStream {

    #[allow(non_snake_case)]
    let GenerateStructErrorCfg { error_type_name, use_bt_types, BacktraceCell, .. } =
        GenerateStructErrorCfg::new(ast);

    let err_impl_with_source = quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl #error_type_name {
            #[inline]
            pub fn new(kind: ErrorKind) -> Self {
                Self::with_backtrace(kind)
            }
            pub fn with_backtrace(kind: ErrorKind) -> Self {
                Self { kind, backtrace: #BacktraceCell ::capture_backtrace(), source: ErrorSource::NoSource }
            }
            pub fn without_backtrace(kind: ErrorKind) -> Self {
                // Really is not needed there, but let's keep it just to test usage of generated 'use'.
                #use_bt_types

                Self { kind, backtrace: #BacktraceCell ::empty(), source: ErrorSource::NoSource }
            }
            pub fn with_source(kind: ErrorKind, source: ErrorSource) -> Self {
                Self { kind, backtrace: #BacktraceCell ::inherit_or_capture(&source), source }
            }
            pub fn with_from<ES: Into<ErrorSource>>(kind: ErrorKind, source: ES) -> Self {
                let src = source.into();
                Self { kind, backtrace: #BacktraceCell ::inherit_or_capture(&src), source: src }
            }
        }

    };

    err_impl_with_source
}


fn generate_struct_error_backtrace_source_impl_block(ast: &syn::DeriveInput)
    -> proc_macro2::TokenStream {

    #[allow(non_snake_case)]
    let GenerateStructErrorCfg { error_type_name, BacktraceSource, BacktraceCell, .. } =
        GenerateStructErrorCfg::new(ast);

    let err_backtrace_provider_impl = quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl #BacktraceSource for #error_type_name {
            #[inline]
            fn contains_backtrace(&self) -> bool { !self.backtrace.is_empty() }
            #[inline]
            fn backtrace_ref(&self) -> Option<& #BacktraceCell> { Some(&self.backtrace) }
            #[inline]
            fn is_taking_backtrace_supported(&self) -> bool { !self.backtrace.is_empty() }
            #[inline]
            fn take_backtrace(&self) -> #BacktraceCell { self.backtrace.move_out() }
        }
    };

    err_backtrace_provider_impl
}


fn generate_struct_error_display_impl_block(ast: &syn::DeriveInput)
    -> proc_macro2::TokenStream {

    #[allow(non_snake_case)]
    let GenerateStructErrorCfg { error_type_name, .. } =
        GenerateStructErrorCfg::new(ast);

    let err_display_impl = quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl core::fmt::Display for #error_type_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, concat!(stringify!(#error_type_name), " {{ {} }}"), self.kind)
            }
        }
    };

    err_display_impl
}


fn generate_struct_error_with_source_debug_impl_block(ast: &syn::DeriveInput)
    -> proc_macro2::TokenStream {

    #[allow(non_snake_case)]
    let GenerateStructErrorCfg { error_type_name, root_type_path, .. } =
        GenerateStructErrorCfg::new(ast);

    let err_debug_impl_with_source = quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl core::fmt::Debug for #error_type_name {

            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                #root_type_path ::error::__private::error_debug_fmt_impl(
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

    err_debug_impl_with_source
}


fn generate_struct_error_without_source_debug_impl_block(ast: &syn::DeriveInput)
    -> proc_macro2::TokenStream {

    #[allow(non_snake_case)]
    let GenerateStructErrorCfg { error_type_name, .. } =
        GenerateStructErrorCfg::new(ast);

    let err_debug_impl_without_source = quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl core::fmt::Debug for #error_type_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                if self.backtrace.is_empty() {
                    write!(f, concat!(stringify!(#error_type_name), " {{ kind: {:?} }}"), self.kind)
                } else {
                    write!(f, concat!(stringify!(#error_type_name), " {{ kind: {:?}, backtrace: {} }}"), self.kind, self.backtrace)
                }
            }
        }
    };

    err_debug_impl_without_source
}


//--------------------------------------------------------------------------------------------------
//                            Struct Error Source enum
//--------------------------------------------------------------------------------------------------
//
pub(crate) fn impl_struct_error_source(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let _name = ast.ident.to_string();

    let StaticStructErrorSourceAttrs {
        do_not_generate_debug, do_not_generate_display, do_not_generate_std_error, ..} =
        StaticStructErrorSourceAttrs::new(ast);

    let err_src_impl = generate_error_source_backtrace_source_impl_block(ast);

    let debug_err_src_impl =
        if do_not_generate_debug { quote!() }
        else { generate_error_source_debug_impl_block(ast) };

    let from_impl: proc_macro2::TokenStream = generate_error_source_variants_from_impl_block(ast);

    let into_impl = generate_error_source_variants_from_args_impl_block(ast);

    let display_err_src_impl =
        if do_not_generate_display { quote!() }
        else { generate_error_source_display_impl_block(ast) };

    let std_error_err_src_impl =
        if do_not_generate_std_error { quote!() }
        else { generate_error_source_std_error_impl_block(ast) };

    let err_impl_ts: proc_macro2::TokenStream = err_src_impl;

    // as separate var to avoid warn/error in RustRover
    let out = quote! {
        #err_impl_ts
        #into_impl
        #from_impl
        #debug_err_src_impl
        #display_err_src_impl
        #std_error_err_src_impl
    };

    out.into()
}


struct ErrorSourceVariants<'a> {
    error_source_enum: ErrorSourceEnum<'a>,
    grouped_err_enum_variants_by_arg_type: HashMap<String, Vec<ErrorSourceEnumVariant<'a>>>,
    duplicated_err_enum_src_types: HashSet<String>,
}
impl ErrorSourceVariants<'_> {

    fn new<'a>(ast: &'a syn::DeriveInput) -> ErrorSourceVariants<'a> {

        let error_source_enum: ErrorSourceEnum<'a> = get_error_source_enum_variants(ast);

        let grouped_err_enum_variants_by_arg_type: HashMap<String, Vec<ErrorSourceEnumVariant<'a>>> =
            error_source_enum
                .variants
                .iter()
                .filter_map(|vr| vr.first_arg_type.map(|first_arg_type| (type_to_string(first_arg_type), vr.clone()) ))
                .into_group_map();

        let duplicated_err_enum_src_types: HashSet<String> = keys_with_duplicates(&grouped_err_enum_variants_by_arg_type);

        ErrorSourceVariants::<'a> {
            error_source_enum,
            grouped_err_enum_variants_by_arg_type,
            duplicated_err_enum_src_types,
        }
    }
}


static SIMPLE_TYPES: [&'static str;18] = [
    "char",
    "i8", "i16", "i32", "i64", "i128",
    "u8", "u16", "u32", "u64", "u128",
    "usize", "isize",
    "f32", "f64",
    "String", "&str", "& 'static str",
];

static TYPES_WITHOUT_ANY_BT: [&'static str;18] = SIMPLE_TYPES;

fn generate_error_source_backtrace_source_impl_block(ast: &syn::DeriveInput)
    -> proc_macro2::TokenStream {

    #[allow(non_snake_case)]
    let GenerateStructErrorCfg { BacktraceSource, BacktraceCell, .. } =
        GenerateStructErrorCfg::new(ast);

    let err_src_vars = ErrorSourceVariants::new(ast);
    let enum_variants = &err_src_vars.error_source_enum.variants;

    let err_src_bt_ref_impl_match_branches: Vec<proc_macro2::TokenStream> = enum_variants.iter().map(|vr|{
        let var_name = vr.name;
        let no_source_backtrace = find_enum_variant_attr(vr.variant, "no_source_backtrace").is_some();
        let is_arg_present = vr.first_arg_type.is_some();

        let is_arg_without_bt: bool = vr.first_arg_type
            .map(|arg_type| TYPES_WITHOUT_ANY_BT.contains(&type_to_string_without_spaces(arg_type).as_str()))
            .unwrap_or(false);

        if !is_arg_present {
            quote!(  ErrorSource:: #var_name  => { None }     )
        } else if no_source_backtrace || is_arg_without_bt {
            quote!(  ErrorSource:: #var_name (_)  => { None }     )
        } else {
            quote!(  ErrorSource:: #var_name (ref src)  => { src.backtrace_ref() }  )
        }
    }).collect::<Vec<_>>();


    let err_src_contains_bt_impl_match_branches: Vec<proc_macro2::TokenStream> = enum_variants.iter().map(|vr|{
        let var_name = vr.name;
        let no_source_backtrace = find_enum_variant_attr(vr.variant, "no_source_backtrace").is_some();
        let is_arg_present = vr.first_arg_type.is_some();

        let is_arg_without_bt: bool = vr.first_arg_type
            .map(|arg_type| TYPES_WITHOUT_ANY_BT.contains(&type_to_string_without_spaces(arg_type).as_str()))
            .unwrap_or(false);

        if !is_arg_present {
            quote!(  ErrorSource:: #var_name  => { false }  )
        } else if no_source_backtrace || is_arg_without_bt {
            quote!(  ErrorSource:: #var_name (_)  => { false }  )
        } else {
            quote!(  ErrorSource:: #var_name (ref src)  => { src.contains_backtrace() }  )
        }
    }).collect::<Vec<_>>();


    let err_src_is_taking_backtrace_supported_impl_match_branches: Vec<proc_macro2::TokenStream> = enum_variants.iter().map(|vr|{
        let var_name = vr.name;
        let no_source_backtrace = find_enum_variant_attr(vr.variant, "no_source_backtrace").is_some();
        let is_arg_present = vr.first_arg_type.is_some();

        let is_arg_without_bt: bool = vr.first_arg_type
            .map(|arg_type| TYPES_WITHOUT_ANY_BT.contains(&type_to_string_without_spaces(arg_type).as_str()))
            .unwrap_or(false);

        if !is_arg_present {
            quote!(  ErrorSource:: #var_name  => { false }  )
        } else if no_source_backtrace || is_arg_without_bt {
            quote!(  ErrorSource:: #var_name (_)  => { false }  )
        } else {
            quote!(  ErrorSource:: #var_name (ref src)  => { src.is_taking_backtrace_supported() }  )
        }
    }).collect::<Vec<_>>();


    let err_src_impl = quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl #BacktraceSource for ErrorSource {
            fn backtrace_ref(&self) -> Option<& #BacktraceCell> {
                use #BacktraceCell;
                match self {
                    #(#err_src_bt_ref_impl_match_branches)*
                    // ErrorSource::NoSource => { BacktraceInfo::empty() }
                    // ErrorSource::ParseBigDecimalError(_)  => { BacktraceInfo::empty()  }
                    // ErrorSource::CurrencyFormatError(ref src) => { src.provide_backtrace() }
                }
            }
            fn contains_backtrace(&self) -> bool {
                // if !self.backtrace.is_empty() {
                //     return true;
                // }

                use #BacktraceCell;
                match self {
                    #(#err_src_contains_bt_impl_match_branches)*
                    // ErrorSource::NoSource => false,
                    // ErrorSource::ParseBigDecimalError(_) => false,
                    // ErrorSource::CurrencyFormatError(ref src) => { src.contains_backtrace() }
                }
            }
            fn is_taking_backtrace_supported(&self) -> bool {

                use #BacktraceSource;
                match self {
                    #(#err_src_is_taking_backtrace_supported_impl_match_branches)*
                }
            }
        }
    };

    err_src_impl
}


fn generate_error_source_debug_impl_block(ast: &syn::DeriveInput)
    -> proc_macro2::TokenStream {

    let err_src_vars = ErrorSourceVariants::new(ast);
    let enum_variants = &err_src_vars.error_source_enum.variants;

    let err_src_debug_impl_match_branches: Vec<proc_macro2::TokenStream> = enum_variants.iter().map(|vr|{
        let var_name = vr.name;
        let is_arg_present = vr.first_arg_type.is_some();

        let to_wrap_with_enum_name: bool = vr.first_arg_type
            .map(|arg_type| TYPES_TO_WRAP_WITH_ENUM_VR_NAME.contains(&type_to_string_without_spaces(arg_type).as_str()))
            .unwrap_or(false);

        if is_arg_present && to_wrap_with_enum_name {
            quote!(  #var_name(ref src)  => { write!(f, concat!(stringify!(#var_name),"({:?})"), src) }  )
        } else if is_arg_present {
            quote!(  #var_name(ref src)  => { write!(f, "{:?}", src) }  )
        } else {
            quote!(  #var_name  => { write!(f, stringify!(#var_name)) }  )
        }
    }).collect::<Vec<_>>();

    let debug_err_src_impl = quote! {

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

    };

    debug_err_src_impl
}


fn generate_error_source_variants_from_impl_block(ast: &syn::DeriveInput)
    -> proc_macro2::TokenStream {

    #[allow(non_snake_case)]
    let StaticStructErrorSourceAttrs { struct_error_type, .. } =
        StaticStructErrorSourceAttrs::new(ast);

    let err_src_vars = ErrorSourceVariants::new(ast);
    let enum_variants = &err_src_vars.error_source_enum.variants;
    let ErrorSourceVariants { grouped_err_enum_variants_by_arg_type, ..} =
        ErrorSourceVariants::new(ast);

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
        let all_enum_vars_for_this_from_err_type: &Vec<ErrorSourceEnumVariant<'_>> = grouped_err_enum_variants_by_arg_type
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
            // impl From<CurrencyFormatError> for AmountFormatError {
            //     fn from(error: CurrencyFormatError) -> Self { AmountFormatError::with_from(ErrorKind::IncorrectCurrency, error) }
            // }

            #[allow(unused_imports)]
            #[allow(unused_qualifications)]
            impl From< #from_err_type_name > for #err_struct_name {
                fn from(error: #from_err_type_name ) -> Self { #err_struct_name::with_from(ErrorKind:: #err_struct_kind_name, error) }
            }
        }
    }).collect::<Vec<_>>();

    let from_expressions = quote! { #(#from_impl)* };
    from_expressions
}


static TYPES_TO_WRAP_WITH_ENUM_VR_NAME: [&'static str; 18] = SIMPLE_TYPES;

fn generate_error_source_display_impl_block(ast: &syn::DeriveInput)
    -> proc_macro2::TokenStream {

    let err_src_vars = ErrorSourceVariants::new(ast);
    let enum_variants = &err_src_vars.error_source_enum.variants;

    // let display_err_src_impl = if do_not_generate_display { quote!() }
    let display_err_src_items_impl: Vec<proc_macro2::TokenStream> = enum_variants.iter().map(|ref el| {
        let var_name: &syn::Ident = el.name;
        let is_src_arg_present: bool = el.first_arg_type.is_some();
        let to_wrap_with_enum_name: bool = el.first_arg_type
            .map(|arg_type| TYPES_TO_WRAP_WITH_ENUM_VR_NAME.contains(&type_to_string_without_spaces(arg_type).as_str()))
            .unwrap_or(false);

        if is_src_arg_present && to_wrap_with_enum_name {
            quote! { ErrorSource:: #var_name (ref src)  => { write!(f, concat!(stringify!(#var_name), "({})"), src) } }
        } else if is_src_arg_present {
            quote! { ErrorSource:: #var_name (ref src)  => { write!(f, "{}", src) } }
        } else {
            quote! { ErrorSource:: #var_name  => { write!(f, stringify!(#var_name)) } }
        }
    }).collect::<Vec<_>>();

    let display_impl = quote! {
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
    };

    display_impl
}


static TYPES_WITHOUT_STD_ERR: [&str; 18] = SIMPLE_TYPES;

fn generate_error_source_std_error_impl_block(ast: &syn::DeriveInput)
    -> proc_macro2::TokenStream {

    let err_src_vars = ErrorSourceVariants::new(ast);
    let enum_variants = &err_src_vars.error_source_enum.variants;

    let fn_source_items_impl: Vec<proc_macro2::TokenStream> = enum_variants.iter().map(|ref el| {
        let var_name: &syn::Ident = el.name;
        let is_src_arg_present: bool = el.first_arg_type.is_some();
        let arg_str_type: Option<String> = el.first_arg_type
            .map(|arg_type| type_to_string_without_spaces(arg_type));
        let no_std_error = find_enum_variant_attr(el.variant, "no_std_error").is_some();
        let no_std_err_for_arg = el.first_arg_type
            .map(|arg_type| type_to_string(arg_type))
            .map(|arg_type_as_str| TYPES_WITHOUT_STD_ERR.contains(&arg_type_as_str.as_str()))
            .unwrap_or(false);

        if !is_src_arg_present {
            quote! { ErrorSource:: #var_name => { None }  }
        } else if no_std_err_for_arg || no_std_error {
            quote! { ErrorSource:: #var_name (_) => { None } }
        } else if arg_str_type.is_eq_to_str("anyhow::Error") {
            quote! { ErrorSource:: #var_name (ref src) => { Some(src.as_ref()) } }
        } else if arg_str_type.is_eq_to_one_of_str([
            "Box<dyn std::error::Error>", "Box<dyn error::Error>", "Box<dyn Error>"]) {
            quote! { ErrorSource:: #var_name (ref src) => { Some(src.as_ref()) } }
        } else {
            // T O D O: could we use some universal way with something like TryAsRef
            // quote! { ErrorSource:: #var_name (ref src) => { Some(src.as_ref()) } }
            quote! { ErrorSource:: #var_name (ref src) => { Some(src) } }
        }
    }).collect::<Vec<_>>();

    let fn_description_items_impl: Vec<proc_macro2::TokenStream> = enum_variants.iter().map(|ref el| {
        let var_name: &syn::Ident = el.name;
        let no_std_error = find_enum_variant_attr(el.variant, "no_std_error").is_some();
        let is_src_arg_present: bool = el.first_arg_type.is_some();
        let no_std_err_for_arg = el.first_arg_type
            .map(|arg_type| type_to_string(arg_type))
            .map(|arg_type_as_str| TYPES_WITHOUT_STD_ERR.contains(&arg_type_as_str.as_str()))
            .unwrap_or(false);

        if is_src_arg_present {
            if no_std_err_for_arg || no_std_error {
                quote! { ErrorSource:: #var_name (_)  => { "" } }
            } else {
                quote! { ErrorSource:: #var_name (ref src)  => { src.description() } }
            }
        } else {
            quote! { ErrorSource:: #var_name => { "" } }
        }
    }).collect::<Vec<_>>();

    let std_error_impl = quote! {
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

    };

    std_error_impl
}


fn generate_error_source_variants_from_args_impl_block(ast: &syn::DeriveInput)
    -> proc_macro2::TokenStream {

    let err_src_vars = ErrorSourceVariants::new(ast);
    let enum_variants = &err_src_vars.error_source_enum.variants;
    let ErrorSourceVariants { error_source_enum, duplicated_err_enum_src_types, ..} =
        ErrorSourceVariants::new(ast);

    let into_impl: Vec<proc_macro2::TokenStream> = enum_variants.iter().filter_map(|ref el| {
        let var_name: &syn::Ident = el.name;

        if el.first_arg_type.is_none() {
            return None;
        }

        let var_arg_type: &syn::Type = el.first_arg_type
            .expect(&format!("first_arg_type is expected for {}.", var_name));

        let arg_type_as_string = type_to_string_without_spaces(var_arg_type);
        if duplicated_err_enum_src_types.contains(&arg_type_as_string) {
            compile_log_info!("'Into/From' is not implemented for {}.{} because there are others enum variants \
            in [{}] with the same src/arg type [{}].",
                error_source_enum.name, var_name, error_source_enum.name, arg_type_as_string);
            return None;
        }

        Some( quote! {
            // impl From<CurrencyFormatError> for ErrorSource {
            //     fn from(CurrencyFormatError) -> ErrorSource { ErrorSource::CurrencyFormatError22(self) }
            // }

            #[allow(unused_imports)]
            #[allow(unused_qualifications)]
            impl From<#var_arg_type> for ErrorSource {
                fn from(v: #var_arg_type) -> ErrorSource { ErrorSource:: #var_name (v) }
            }
        })
    }).collect::<Vec<_>>();

    let into = quote! { #(#into_impl)* };
    into
}
