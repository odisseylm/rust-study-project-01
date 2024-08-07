use std::collections::{ HashMap, HashSet };
use itertools::Itertools;
use quote::quote;
use crate::bt::{
    BacktraceTypeKind,
    bt_root_path_segment, bt_type, get_bt_type_kind, is_bt_type, use_bt_types_expr,
};
use crate::error_source::get_internal_type_path_mode;
use crate::macro_util::{
    type_to_string, keys_with_duplicates, has_one_of_attrs,
    InternalTypePathMode, SIMPLE_TYPES,
};
use crate::this_error_bt_src_ext::generate_bt_source;
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
pub(crate) struct ThisErrorExtCfg {
    pub(crate) _error_type_name: proc_macro2::Ident,
    #[allow(dead_code)]
    pub(crate) internal_type_path_mode: InternalTypePathMode,
    // expressions
    pub(crate) bt_root_type_path: proc_macro2::TokenStream,
    pub(crate) _use_bt_types: proc_macro2::TokenStream,
    pub(crate) BacktraceCell: proc_macro2::TokenStream,
    pub(crate) BacktraceSource: proc_macro2::TokenStream,
}
impl ThisErrorExtCfg {
    pub(crate) fn new(ast: &syn::DeriveInput) -> Self {
        let internal_type_path_mode = get_internal_type_path_mode(ast);
        Self {
            internal_type_path_mode,
            _error_type_name: ast.ident.clone(),
            // expressions
            bt_root_type_path: bt_root_path_segment(internal_type_path_mode),
            _use_bt_types: use_bt_types_expr(internal_type_path_mode),
            BacktraceCell: bt_type(internal_type_path_mode, "BacktraceCell"),
            BacktraceSource: bt_type(internal_type_path_mode, "BacktraceSource"),
        }
    }
}


pub(crate) struct ThisErrorEnumVariants<'a> {
    pub(crate) error_source_enum: ThisErrorEnum<'a>,
    pub(crate) _grouped_err_enum_variants_by_arg_types: HashMap<String, Vec<ThisErrorEnumVariant<'a>>>,
    pub(crate) duplicated_err_enum_src_types: HashSet<String>,
}
impl ThisErrorEnumVariants<'_> {

    pub(crate) fn new<'a>(ast: &'a syn::DeriveInput) -> ThisErrorEnumVariants<'a> {

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

pub(crate) fn impl_this_error_bt_src(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let bt_source_impl = generate_bt_source(ast);
    let code = quote! { #bt_source_impl };
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

    let into_impl: Vec<proc_macro2::TokenStream> = vars_for_from.iter()
        .filter_map(|el| {

            let var_name: &syn::Ident = el.name;

            if el.all_arg_types.is_empty() {
                return None;
            }

            if el.arg_types_without_bt.len() != 1 {
                panic!("'From' can be generated only for enum variant with one non-backtrace arg. \
                        But [{var_name}] has {}.", el.arg_types_without_bt.len());
            }

            let arg_entry = el.arg_types_without_bt.first()
                .expect(&format!("Incorrect enum variant [{var_name}]"));

            let var_arg_type: &syn::Type = arg_entry.1;
            let arg_type_as_string = type_to_string(var_arg_type);

            if duplicated_err_enum_src_types.contains(&arg_type_as_string) {
                panic!(
                    "'Into/From' is not implemented for {}.{} because there are others enum variants \
                      in [{}] with the same src/arg type [{}].",
                    error_source_enum.name, var_name, error_source_enum.name, arg_type_as_string);
            }

            let inherit_or_capture = has_one_of_attrs(
                &el._variant.attrs, ["inherit_or_capture", "inherit_or_capture_bt"]);

            let code = match arg_entry.0 {
                ArgName::Index(ref _tuple_arg_index) => {
                    if inherit_or_capture {
                        quote! {
                            #[allow(unused_imports)]
                            #[allow(unused_qualifications)]
                            impl From<#var_arg_type> for #this_err_enum_name {
                                fn from(v: #var_arg_type) -> #this_err_enum_name {
                                    let bt = #BacktraceCell :: inherit_or_capture(&v);
                                    #this_err_enum_name :: #var_name (
                                        v,
                                        bt.into(),
                                    )
                                }
                            }
                        }

                    } else {
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
                }

                ArgName::Ident(ref arg_name) => {
                    if inherit_or_capture {
                        quote! {
                            #[allow(unused_imports)]
                            #[allow(unused_qualifications)]
                            impl From<#var_arg_type> for #this_err_enum_name {
                                fn from(v: #var_arg_type) -> #this_err_enum_name {
                                    let bt = #BacktraceCell :: inherit_or_capture(&v);
                                    #this_err_enum_name :: #var_name {
                                        #arg_name: v,
                                        backtrace: bt.into(),
                                    }
                                }
                            }
                        }
                    } else {
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
    pub(crate) bt_src_kind: BtSrcErrorEnumVariantKind,
}
#[derive(Debug)]
pub(crate) struct ThisErrorEnum<'a> {
    pub(crate) name: &'a syn::Ident,
    pub(crate) variants: Vec<ThisErrorEnumVariant<'a>>,
}


#[derive(Debug, Clone)]
pub(crate) enum BtSrcErrorEnumVariantKind {
    BacktraceCell(ArgName),
    StdBacktrace(ArgName),
    // We should consider it as BacktraceSource by default.
    // To skip/ignore by annotation.
    Unknown(ArgName),
    SimpleType(ArgName),
    AnyhowError(ArgName),
    // We cannot determine it. Macro annotation should be used.
    StdError(ArgName),
    // Controlled by annotation.
    Skip(ArgName),
}


fn has_from_with_bt_attr(attrs: &Vec<syn::Attribute>) -> bool {
    has_one_of_attrs(attrs, ["from_bt", "from_with_bt"])
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
                        bt_src_kind: BtSrcErrorEnumVariantKind::Skip(ArgName::Index(0)),
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

    let bt_arg = bt_arg_type(&all_arg_types);
    let non_bt_args = non_bt_arg_type(&all_arg_types);
    let non_bt_arg = non_bt_args.first();
    let arg_types_without_bt = arg_types_without_bt(&all_arg_types);
    let arg_types_without_bt_as_str = arg_types_as_str(&arg_types_without_bt);
    let variant_name: &syn::Ident = &enum_variant.ident;

    let skip_bt_source = has_one_of_attrs(
        &enum_variant.attrs, ["skip_bt_source", "no_source_backtrace"]);
    let std_error_source = has_one_of_attrs(
        &enum_variant.attrs, ["std_error_bt_source", "std_error"]);

    let bt_src_kind: BtSrcErrorEnumVariantKind =
        if skip_bt_source {
            BtSrcErrorEnumVariantKind::Skip(ArgName::Index(0))
        } else if let Some(ref bt_arg) = bt_arg {
            let (arg_name, arg_type) = bt_arg;
            let bt_type_kind = get_bt_type_kind(bt_arg.1);
            match bt_type_kind {
                None => {
                    panic!("Unexpected backtrace type [{}] (in variant [{variant_name}/{arg_name:?}])",
                           type_to_string(arg_type))
                }
                Some(BacktraceTypeKind::BacktraceCell) => {
                    BtSrcErrorEnumVariantKind::BacktraceCell(arg_name.clone())
                }
                Some(BacktraceTypeKind::StdBacktrace) => {
                    BtSrcErrorEnumVariantKind::StdBacktrace(arg_name.clone())
                }
            }
        } else if non_bt_args.len() != 1 {
            BtSrcErrorEnumVariantKind::Skip(ArgName::Index(0))
        } else if let Some((non_bt_arg, non_bt_arg_type)) = non_bt_arg {
            let non_bt_arg_type_str = type_to_string(non_bt_arg_type);
            let non_bt_arg_type_str = non_bt_arg_type_str.as_str();

            if std_error_source {
                BtSrcErrorEnumVariantKind::StdError(non_bt_arg.clone())
            } else if non_bt_arg_type_str == "anyhow::Error" {
                BtSrcErrorEnumVariantKind::AnyhowError(non_bt_arg.clone())
            } else if SIMPLE_TYPES.contains(&non_bt_arg_type_str) {
                BtSrcErrorEnumVariantKind::SimpleType(non_bt_arg.clone())
            } else {
                BtSrcErrorEnumVariantKind::Unknown(non_bt_arg.clone())
            }
        } else {
            BtSrcErrorEnumVariantKind::Skip(ArgName::Index(0))
        };

    ThisErrorEnumVariant::<'a> {
        _variant: enum_variant, name: variant_name,
        all_arg_types, arg_types_without_bt, arg_types_without_bt_as_str,
        is_from_with_bt, bt_src_kind,
    }
}

fn arg_types_without_bt<'a>(all_arg_types: &Vec<(ArgName, &'a syn::Type)>) -> Vec<(ArgName, &'a syn::Type)> {
    all_arg_types.iter()
        .filter(|e| !is_bt_type(e.1))
        .map(|t|t.clone())
        .collect::<Vec<_>>()
}

fn bt_arg_type<'a>(all_arg_types: &Vec<(ArgName, &'a syn::Type)>) -> Option<(ArgName, &'a syn::Type)> {
    all_arg_types.iter()
        .find(|e| is_bt_type(e.1))
        .map(|t|t.clone())
}

fn non_bt_arg_type<'a>(all_arg_types: &Vec<(ArgName, &'a syn::Type)>) -> Vec<(ArgName, &'a syn::Type)> {
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
