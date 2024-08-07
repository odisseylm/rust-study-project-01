use crate::macro_util::make_ident;
use quote::quote;
use crate::this_error_ext::{
    ArgName, BtSrcErrorEnumVariantKind, ThisErrorEnumVariants, ThisErrorExtCfg,
};
//--------------------------------------------------------------------------------------------------


fn tuple_pattern(var_name: &str, index: usize) -> proc_macro2::TokenStream {
    let var_name = make_ident(var_name);
    match index {
        0 => quote! { ref #var_name, .. },
        1 => quote! { _, ref #var_name, .. },
        2 => quote! { _, _, ref #var_name, .. },
        3 => quote! { _, _, _, ref #var_name, .. },
        4 => quote! { _, _, _, _, ref #var_name, .. },
        5 => quote! { _, _, _, _, _, ref #var_name, .. },
        _ => panic!("Unsupported tuple index {index}"),
    }
}

pub(crate) fn generate_bt_source(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {

    let this_err_enum = ThisErrorEnumVariants::new(ast);
    let ThisErrorEnumVariants { error_source_enum, .. } = this_err_enum;
    let enum_name = error_source_enum.name;

    #[allow(non_snake_case)]
    let ThisErrorExtCfg { BacktraceSource, bt_root_type_path: root, .. } = ThisErrorExtCfg::new(ast);

    let variants = &error_source_enum.variants;

    let mut backtrace_ref_matches = Vec::<proc_macro2::TokenStream>::new();
    let mut contains_backtrace_matches = Vec::<proc_macro2::TokenStream>::new();
    let mut is_taking_backtrace_supported_matches = Vec::<proc_macro2::TokenStream>::new();

    for vr in variants {
        let vr_name = &vr._variant.ident;

        match vr.bt_src_kind {
            BtSrcErrorEnumVariantKind::BacktraceCell(ref arg_name) => {
                match arg_name {
                    ArgName::Ident(ref struct_field) => {
                        backtrace_ref_matches.push(quote! {
                            #enum_name :: #vr_name { ref #struct_field, .. } =>
                                { #struct_field .backtrace_ref() }
                        });
                        contains_backtrace_matches.push(quote! {
                            #enum_name :: #vr_name { ref #struct_field, .. } =>
                                { #struct_field .contains_backtrace() }
                        });
                        is_taking_backtrace_supported_matches.push(quote! {
                            #enum_name :: #vr_name { ref #struct_field, .. } =>
                                { #struct_field .is_taking_backtrace_supported() }
                        });
                    }
                    ArgName::Index(tuple_index) => {
                        let pattern = tuple_pattern("backtrace", *tuple_index);

                        backtrace_ref_matches.push(quote! {
                            #enum_name :: #vr_name ( #pattern ) =>
                                { backtrace.backtrace_ref() }
                        });
                        contains_backtrace_matches.push(quote! {
                            #enum_name :: #vr_name ( #pattern ) =>
                                { backtrace.contains_backtrace() }
                        });
                        is_taking_backtrace_supported_matches.push(quote! {
                            #enum_name :: #vr_name ( #pattern ) =>
                                { backtrace.is_taking_backtrace_supported() }
                        });
                    }
                }
            }
            BtSrcErrorEnumVariantKind::StdBacktrace(ref arg_name) => {
                match arg_name {
                    ArgName::Ident(ref struct_field) => {
                        backtrace_ref_matches.push(quote! {
                            #enum_name :: #vr_name { ref #struct_field, .. } => { None }
                        });
                        contains_backtrace_matches.push(quote! {
                            #enum_name :: #vr_name { ref #struct_field, .. } =>
                                { #struct_field.status() == std::backtrace::BacktraceStatus::Captured }
                        });
                        is_taking_backtrace_supported_matches.push(quote! {
                            #enum_name :: #vr_name { ref #struct_field, .. } => { false }
                        });
                    }
                    ArgName::Index(ref tuple_index) => {
                        let pattern = tuple_pattern("backtrace", *tuple_index);

                        backtrace_ref_matches.push(quote! {
                            #enum_name :: #vr_name ( #pattern ) => { None }
                        });
                        contains_backtrace_matches.push(quote! {
                            #enum_name :: #vr_name ( #pattern  ) =>
                                { backtrace.status() == std::backtrace::BacktraceStatus::Captured }
                        });
                        is_taking_backtrace_supported_matches.push(quote! {
                            #enum_name :: #vr_name ( #pattern  ) => { false }
                        });
                    }
                }
            }
            BtSrcErrorEnumVariantKind::Unknown(ref arg_name) => {
                match arg_name {
                    ArgName::Ident(ref struct_field) => {
                        backtrace_ref_matches.push(quote! {
                            #enum_name :: #vr_name { ref #struct_field, .. } =>
                                { #struct_field . backtrace_ref() }
                        });
                        contains_backtrace_matches.push(quote! {
                            #enum_name :: #vr_name { ref #struct_field, .. } =>
                                { #struct_field . contains_backtrace() }
                        });
                        is_taking_backtrace_supported_matches.push(quote! {
                            #enum_name :: #vr_name { ref #struct_field, .. } =>
                                { #struct_field . is_taking_backtrace_supported() }
                        });
                    }
                    ArgName::Index(ref tuple_index) => {
                        let pattern = tuple_pattern("unknown_err", *tuple_index);

                        backtrace_ref_matches.push(quote! {
                            #enum_name :: #vr_name ( #pattern ) =>
                                { unknown_err .backtrace_ref() }
                        });
                        contains_backtrace_matches.push(quote! {
                            #enum_name :: #vr_name ( #pattern ) =>
                                { unknown_err .contains_backtrace() }
                        });
                        is_taking_backtrace_supported_matches.push(quote! {
                            #enum_name :: #vr_name ( #pattern ) =>
                                { unknown_err .is_taking_backtrace_supported() }
                        });
                    }
                }
            }
            BtSrcErrorEnumVariantKind::SimpleType(ref arg_name) => {
                match arg_name {
                    ArgName::Ident(ref _struct_field) => {
                        backtrace_ref_matches.push(quote! {
                            #enum_name :: #vr_name { .. } => { None }
                        });
                        contains_backtrace_matches.push(quote! {
                            #enum_name :: #vr_name { .. } => { false }
                        });
                        is_taking_backtrace_supported_matches.push(quote! {
                            #enum_name :: #vr_name { .. } => { false }
                        });
                    }
                    ArgName::Index(ref tuple_index) => {
                        let pattern = tuple_pattern("_simple_val", *tuple_index);

                        backtrace_ref_matches.push(quote! {
                            #enum_name :: #vr_name ( #pattern ) => { None }
                        });
                        contains_backtrace_matches.push(quote! {
                            #enum_name :: #vr_name ( #pattern ) => { false }
                        });
                        is_taking_backtrace_supported_matches.push(quote! {
                            #enum_name :: #vr_name ( #pattern ) => { false }
                        });
                    }
                }
            }
            BtSrcErrorEnumVariantKind::AnyhowError(ref arg_name) => {
                match arg_name {
                    ArgName::Ident(ref struct_field) => {
                        backtrace_ref_matches.push(quote! {
                            #enum_name :: #vr_name { ref #struct_field, .. } =>
                                { #struct_field .backtrace_ref() }
                        });
                        contains_backtrace_matches.push(quote! {
                            #enum_name :: #vr_name { ref #struct_field, .. } =>
                                { #struct_field .contains_backtrace() }
                        });
                        is_taking_backtrace_supported_matches.push(quote! {
                            #enum_name :: #vr_name { ref #struct_field, .. } =>
                                { #struct_field .is_taking_backtrace_supported() }
                        });
                    }
                    ArgName::Index(ref tuple_index) => {
                        let pattern = tuple_pattern("anyhow_error", *tuple_index);

                        backtrace_ref_matches.push(quote! {
                            #enum_name :: #vr_name ( #pattern ) =>
                                { anyhow_error .backtrace_ref() }
                        });
                        contains_backtrace_matches.push(quote! {
                            #enum_name :: #vr_name ( #pattern ) =>
                                { anyhow_error .contains_backtrace() }
                        });
                        is_taking_backtrace_supported_matches.push(quote! {
                            #enum_name :: #vr_name ( #pattern ) =>
                                { anyhow_error .is_taking_backtrace_supported() }
                        });
                    }
                }
            }
            BtSrcErrorEnumVariantKind::StdError(ref arg_name) => {
                match arg_name {
                    ArgName::Ident(ref struct_field) => {
                        backtrace_ref_matches.push(quote! {
                            #enum_name :: #vr_name { ref #struct_field, .. } => {
                                #root ::backtrace:: std_error_backtrace_ref(#struct_field)
                            }
                        });
                        contains_backtrace_matches.push(quote! {
                            #enum_name :: #vr_name { ref #struct_field, .. } => {
                                #root ::backtrace:: std_error_contains_backtrace(#struct_field)
                            }
                        });
                        is_taking_backtrace_supported_matches.push(quote! {
                            #enum_name :: #vr_name { ref #struct_field, .. } => {
                                #root ::backtrace:: std_error_is_taking_backtrace_supported(#struct_field)
                            }
                        });
                    }
                    ArgName::Index(ref tuple_index) => {
                        let pattern = tuple_pattern("std_error", *tuple_index);

                        backtrace_ref_matches.push(quote! {
                            #enum_name :: #vr_name ( #pattern ) => {
                                #root ::backtrace:: std_error_backtrace_ref(std_error)
                            }
                        });
                        contains_backtrace_matches.push(quote! {
                            #enum_name :: #vr_name ( #pattern ) => {
                                #root ::backtrace:: std_error_contains_backtrace(std_error)
                            }
                        });
                        is_taking_backtrace_supported_matches.push(quote! {
                            #enum_name :: #vr_name ( #pattern ) => {
                                #root ::backtrace:: std_error_is_taking_backtrace_supported(std_error)
                            }
                        });
                    }
                }
            }
            BtSrcErrorEnumVariantKind::Skip(ref _arg_name) => {
            }
        }
    }

    let code = quote! {
        impl mvv_common::backtrace::BacktraceSource for ThisError1 {
            #[allow(unused_imports)]
            fn backtrace_ref(&self) -> Option<&BacktraceCell> {
                use #BacktraceSource;
                match self {
                    #(#backtrace_ref_matches)*

                    #[allow(unreachable_patterns)]
                    _ => None
                }
            }

            #[allow(unused_imports)]
            fn contains_backtrace(&self) -> bool {
                use #BacktraceSource;
                match self {
                    #(#contains_backtrace_matches)*

                    #[allow(unreachable_patterns)]
                    _ => false
                }
            }

            #[allow(unused_imports)]
            fn is_taking_backtrace_supported(&self) -> bool {
                use #BacktraceSource;
                match self {
                    #(#is_taking_backtrace_supported_matches)*

                    #[allow(unreachable_patterns)]
                    _ => false
                }
            }
        }
    };
    code
}
