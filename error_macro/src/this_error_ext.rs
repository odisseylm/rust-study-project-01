use std::collections::{ HashMap, HashSet };
use itertools::Itertools;
use quote::quote;
use crate::bt::{bt_root_path_segment, bt_type, use_bt_types_expr };
use crate::error_source::get_internal_type_path_mode;
use crate::macro_util::{
    find_attr, type_to_string,
    InternalTypePathMode, StringOp,
    keys_with_duplicates, type_to_string_without_spaces,
};
//--------------------------------------------------------------------------------------------------

// RustRover does not pick it up (however cargo does)
// #[macro_use]
// mod compile_log_macros;
include!("compile_log_macros.rs");


/*
#[allow(non_snake_case)]
struct ThisErrorExtAttrs {
    do_not_generate_display: bool,
    do_not_generate_debug: bool,
    do_not_generate_std_error: bool,
    struct_error_type: String,
    // struct_error_internal_type_path_mode: InternalTypePathMode,
}
impl ThisErrorExtAttrs {
    fn new(ast: &syn::DeriveInput) -> Self {

        let struct_error_type_attr: Option<&syn::Attribute> = find_attr(&ast.attrs, "struct_error_type");
        let struct_error_type: Option<String> = struct_error_type_attr
            .and_then(|attr| attr_list_as_string(attr));
        let struct_error_type: String = struct_error_type
            .expect("struct_error_type should have format: struct_error_type(MyErrorStructName)");

        Self {
            do_not_generate_display: find_attr( &ast.attrs, "do_not_generate_display").is_some(),
            do_not_generate_debug: find_attr( &ast.attrs, "do_not_generate_debug").is_some(),
            do_not_generate_std_error: find_attr(&ast.attrs, "do_not_generate_std_error").is_some(),
            struct_error_type,
            // struct_error_internal_type_path_mode: ,
        }
    }
}
*/


#[allow(non_snake_case)]
struct ThisErrorExtCfg {
    _error_type_name: proc_macro2::Ident,
    #[allow(dead_code)]
    internal_type_path_mode: InternalTypePathMode,
    // expressions
    _root_type_path: proc_macro2::TokenStream,
    _use_bt_types: proc_macro2::TokenStream,
    BacktraceCell: proc_macro2::TokenStream,
    BacktraceSource: proc_macro2::TokenStream,
}
impl ThisErrorExtCfg {
    fn new(ast: &syn::DeriveInput) -> Self {
        let internal_type_path_mode = get_internal_type_path_mode(ast);
        Self {
            internal_type_path_mode,
            _error_type_name: ast.ident.clone(),
            // expressions
            _root_type_path: bt_root_path_segment(internal_type_path_mode),
            _use_bt_types: use_bt_types_expr(internal_type_path_mode),
            BacktraceCell: bt_type(internal_type_path_mode, "BacktraceCell"),
            BacktraceSource: bt_type(internal_type_path_mode, "BacktraceSource"),
        }
    }
}


struct ThisErrorEnumVariants<'a> {
    error_source_enum: ThisErrorEnum<'a>,
    _grouped_err_enum_variants_by_arg_types: HashMap<String, Vec<ThisErrorEnumVariant<'a>>>,
    duplicated_err_enum_src_types: HashSet<String>,
}
impl ThisErrorEnumVariants<'_> {

    fn new<'a>(ast: &'a syn::DeriveInput) -> ThisErrorEnumVariants<'a> {

        let error_source_enum: ThisErrorEnum<'a> = get_err_enum_variants(ast);

        let grouped_err_enum_variants_by_arg_types: HashMap<String, Vec<ThisErrorEnumVariant<'a>>> =
            error_source_enum
                .variants
                .iter()
                .map(|vr| (vr.arg_types_without_bt_as_str.clone(), vr.clone()) )
                .into_group_map();

        let duplicated_err_enum_src_types: HashSet<String> =
            keys_with_duplicates(&grouped_err_enum_variants_by_arg_types);

        ThisErrorEnumVariants::<'a> {
            error_source_enum,
            _grouped_err_enum_variants_by_arg_types: grouped_err_enum_variants_by_arg_types,
            duplicated_err_enum_src_types,
        }
    }
}


pub(crate) fn impl_this_error_ext(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let from_expr = generate_from_with_bt_expr_for_this_error_ext(ast);

    let code = quote! { #from_expr };
    code.into()
}


fn generate_from_with_bt_expr_for_this_error_ext(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {

    let this_err_enum = ThisErrorEnumVariants::new(ast);
    let ThisErrorEnumVariants { error_source_enum, duplicated_err_enum_src_types, .. } = this_err_enum;
    let this_err_enum_name = error_source_enum.name;

    #[allow(non_snake_case)]
    let ThisErrorExtCfg { BacktraceCell, .. } = ThisErrorExtCfg::new(ast);

    let vars_for_from = error_source_enum.variants.iter()
        .filter(|el|el.is_from_with_bt)
        .collect::<Vec<_>>();

    let into_impl: Vec<proc_macro2::TokenStream> = vars_for_from.iter().filter_map(|ref el| {
        let var_name: &syn::Ident = el.name;

        if el.all_arg_types.is_empty() {
            return None;
        }

        if el.arg_types_without_bt.len() != 1 {
            panic!("'From' can be generated only for enum variant with one non-backtrace arg. \
                    But [{var_name}] has {}.", el.arg_types_without_bt.len());
        }

        let arg_entry = el.all_arg_types.first()
            .expect(&format!("Incorrect enum variant [{var_name}]"));

        let var_arg_type: &syn::Type = arg_entry.1;
        let arg_type_as_string = type_to_string_without_spaces(var_arg_type);

        if duplicated_err_enum_src_types.contains(&arg_type_as_string) {
            panic!(
                "'Into/From' is not implemented for {}.{} because there are others enum variants \
                  in [{}] with the same src/arg type [{}].",
                error_source_enum.name, var_name, error_source_enum.name, arg_type_as_string);
        }

        let code = match arg_entry.0 {
            ArgName::Index(ref _tuple_arg_index) => {
                quote! {
                    #[allow(unused_imports)]
                    #[allow(unused_qualifications)]
                    impl From<#var_arg_type> for #this_err_enum_name {
                        fn from(v: #var_arg_type) -> #this_err_enum_name {
                            #this_err_enum_name :: #var_name (
                                v,
                                #BacktraceCell :: capture_backtrace().into(),
                            )
                        }
                    }
                }

            }
            ArgName::Ident(ref arg_name) => {
                quote! {
                    #[allow(unused_imports)]
                    #[allow(unused_qualifications)]
                    impl From<#var_arg_type> for #this_err_enum_name {
                        fn from(v: #var_arg_type) -> #this_err_enum_name {
                            #this_err_enum_name :: #var_name {
                                #arg_name: v,
                                backtrace: ( #BacktraceCell :: capture_backtrace().into() ),
                            }
                        }
                    }
                }

            }
        };

        Some(code)
    }).collect::<Vec<_>>();

    let into_expressions = quote! { #(#into_impl)* };
    into_expressions
}


#[derive(Debug, Clone)]
pub(crate) enum ArgName {
    Ident(syn::Ident),
    // for tuple
    Index(usize)
}

#[derive(Debug, Clone)]
pub(crate) struct ThisErrorEnumVariant<'a> {
    pub(crate) _variant: &'a syn::Variant,
    pub(crate) name: &'a syn::Ident,
    pub(crate) all_arg_types: Vec<(ArgName, &'a syn::Type)>,
    pub(crate) arg_types_without_bt: Vec<(ArgName, &'a syn::Type)>,
    pub(crate) is_from_with_bt: bool,
    // To find duplicates.
    pub(crate) arg_types_without_bt_as_str: String,
}
#[derive(Debug)]
pub(crate) struct ThisErrorEnum<'a> {
    pub(crate) name: &'a syn::Ident,
    pub(crate) variants: Vec<ThisErrorEnumVariant<'a>>,
}


fn has_from_with_bt_attr(attrs: &Vec<syn::Attribute>) -> bool {
    find_attr(attrs, "from_bt").is_some()
        || find_attr(attrs, "from_with_bt").is_some()
}

fn get_err_enum_variants<'a>(ast: & 'a syn::DeriveInput) -> ThisErrorEnum<'a> {

    let enum_name: &syn::Ident = &ast.ident;
    let mut variants: Vec<ThisErrorEnumVariant<'a>> = Vec::new();

    if let syn::Data::Enum(ref data_enum) = ast.data {

        data_enum.variants.iter().for_each(|vr| {
            let enum_el: &syn::Variant = vr;
            let variant_name: &syn::Ident = &vr.ident;

            let is_from_with_bt = has_from_with_bt_attr(&vr.attrs);

            match &vr.fields {
                syn::Fields::Unit => {
                    variants.push(ThisErrorEnumVariant {
                        _variant: enum_el,
                        name: variant_name,
                        all_arg_types: Vec::new(),
                        arg_types_without_bt: Vec::new(),
                        arg_types_without_bt_as_str: "".to_owned(),
                        is_from_with_bt: false,
                    });
                }
                syn::Fields::Unnamed(ref fields) => {
                    variants.push(
                        gather_this_error_enum_variant_info(
                            enum_el, is_from_with_bt, ||fields.unnamed.iter()));
                }
                syn::Fields::Named(ref fields) => {
                    variants.push(
                        gather_this_error_enum_variant_info(
                            enum_el, is_from_with_bt, ||fields.named.iter()));
                }
            };
        })
    }
    else {
        panic!("Type {} should be enum", enum_name);
    }

    ThisErrorEnum {
        name: enum_name,
        variants,
    }
}

fn gather_this_error_enum_variant_info<'a, IterF > (
    enum_variant: &'a syn::Variant, is_from_with_bt: bool, iter_fn: IterF)
    -> ThisErrorEnumVariant<'a>
    where IterF: Fn() -> syn::punctuated::Iter<'a, syn::Field> {

    let is_from_with_bt =
        if is_from_with_bt { is_from_with_bt }
        else {
            iter_fn().find(|el| has_from_with_bt_attr(&el.attrs)).is_some()
        };

    let all_arg_types = iter_fn()
        .enumerate()
        .map(|(index, f)|
        match f.ident {
            None =>
                (ArgName::Index(index), &f.ty),
            Some(ref arg_ident) =>
                (ArgName::Ident(arg_ident.clone()), &f.ty),
        })
        .collect::<Vec<_>>();

    let arg_types_without_bt = arg_types_without_bt(&all_arg_types);
    let arg_types_without_bt_as_str = arg_types_as_str(&arg_types_without_bt);
    let variant_name: &syn::Ident = &enum_variant.ident;

    ThisErrorEnumVariant::<'a> {
        _variant: enum_variant, name: variant_name,
        all_arg_types, arg_types_without_bt, arg_types_without_bt_as_str,
        is_from_with_bt,
    }
}

fn arg_types_without_bt<'a>(all_arg_types: &Vec<(ArgName, &'a syn::Type)>) -> Vec<(ArgName, &'a syn::Type)> {
    all_arg_types.iter()
        .filter(|e| !is_bt_type(e.1))
        .map(|t|t.clone())
        .collect::<Vec<_>>()
}

fn arg_types_as_str(all_arg_types: &Vec<(ArgName, &syn::Type)>) -> String {
    all_arg_types.iter()
        .map(|e|type_to_string(e.1))
        .join(", ")
}

fn is_bt_type(t: &syn::Type) -> bool {
    let type_str = type_to_string(t);
    let type_str = type_str.as_str();

    if type_str.is_eq_to_one_of_str([
        "Backtrace", "std::backtrace::Backtrace", "BacktraceCell",
        "StdBacktrace", "StdBacktraceAlias", "BacktraceAlias",
    ]) {
        return true;
    }

    if type_str.ends_with_one_of(["::Backtrace", "::StdBacktrace", "::BacktraceCell"]) {
        return true;
    }

    false
}
