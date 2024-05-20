use quote::quote;


#[proc_macro_derive(MyStaticStructError, attributes(StaticStructErrorType))]
pub fn my_static_struct_error_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    // construct a representation of Rust code as a syntax tree that we can manipulate
    let ast: syn::DeriveInput = syn::parse(input)
        .expect("No input for derive macro MyStaticStructError");

    // build the trait implementation
    impl_my_static_struct_error(&ast)
}

fn impl_my_static_struct_error(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let name = &ast.ident;

    let source_field_exists: bool = if let syn::Data::Struct(ref data) = ast.data {
        if let syn::Fields::Named(ref fields) = data.fields {
            fields.named.iter().any(|f|
                f.ident
                .as_ref()
                .map(|ident_name| ident_name == "source")
                .unwrap_or(false)
            )
        }
        // T O D O: how to write it shortly (without 2 else->false)
        else { false }
    } else { false };

    let err_impl_without_source = quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl #name {

            pub fn new(kind: ErrorKind) -> Self {
                use crate::util::backtrace::NewBacktracePolicy;
                use crate::util::BacktraceInfo;
                Self { kind, backtrace: BacktraceInfo::new() }
            }
            pub fn with_backtrace(kind: ErrorKind, backtrace_policy: crate::util::backtrace::NewBacktracePolicy) -> Self {
                use crate::util::backtrace::NewBacktracePolicy;
                use crate::util::BacktraceInfo;
                Self { kind, backtrace: BacktraceInfo::new_by_policy(backtrace_policy) }
            }
        }

    };

    let err_impl_with_source = quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl #name {
            pub fn new(kind: ErrorKind) -> Self {
                use crate::util::backtrace::NewBacktracePolicy;
                use crate::util::BacktraceInfo;
                Self { kind, backtrace: BacktraceInfo::new(), source: ErrorSource::NoSource }
            }
            pub fn with_backtrace(kind: ErrorKind, backtrace_policy: crate::util::backtrace::NewBacktracePolicy) -> Self {
                use crate::util::backtrace::NewBacktracePolicy;
                use crate::util::BacktraceInfo;
                Self { kind, backtrace: BacktraceInfo::new_by_policy(backtrace_policy), source: ErrorSource::NoSource }
            }
            pub fn with_source(kind: ErrorKind, source: ErrorSource) -> Self {
                use crate::util::backtrace::NewBacktracePolicy;
                use crate::util::BacktraceInfo;
                Self { kind, backtrace: BacktraceInfo::inherit_from(&source), source }
            }
            pub fn with_from<ES: Into<ErrorSource>>(kind: ErrorKind, source: ES) -> Self {
                use crate::util::backtrace::NewBacktracePolicy;
                use crate::util::BacktraceInfo;
                let src = source.into();
                Self { kind, backtrace: BacktraceInfo::inherit_from(&src), source: src }
            }
        }

    };

    let err_backtrace_provider_impl = quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl crate::util::backtrace::BacktraceCopyProvider for #name {
            fn provide_backtrace(&self) -> crate::util::BacktraceInfo { self.backtrace.clone() }
        }
    };

    let err_display_impl = quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl core::fmt::Display for #name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "#name  {}", self.kind)
            }
        }
    };

    let err_impl: proc_macro2::TokenStream = if source_field_exists { err_impl_with_source } else { err_impl_without_source };

    // T O D O: probably it can be done in some short standard way ??
    let mut all = proc_macro::TokenStream::new();
    all.add_pm2_ts(err_impl);
    all.add_pm2_ts(err_backtrace_provider_impl);
    all.add_pm2_ts(err_display_impl);
    all
}



#[proc_macro_derive(MyStaticStructErrorSource, attributes(struct_error_type, from_error_kind, no_source_backtrace))]
pub fn my_static_struct_error_source_macro_derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse(input)
        .expect("No input source for derive macro MyStaticStructErrorSource.");
    impl_my_static_struct_error_source(&ast)
}

fn impl_my_static_struct_error_source(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let _name = ast.ident.to_string();

    let struct_error_type_attr: Option<&syn::Attribute> = find_attr(&ast.attrs, "struct_error_type");
    let struct_error_type: Option<String> = struct_error_type_attr
        .and_then(|attr| attr_list_as_string(attr));
    let struct_error_type: String = struct_error_type
        .expect("struct_error_type should have format: struct_error_type(MyErrorStructName)");

    let enum_variants = get_error_source_enum_variants(ast);
    let enum_variants_wo_no_source: Vec<&ErrorSourceEnumVariant> = enum_variants
        .variants.iter()
        .filter(|vr|{ vr.name != "NoSource" })
        .collect::<Vec<_>>();

    let err_src_bt_provider_impl_match_branches: Vec<proc_macro2::TokenStream> = enum_variants_wo_no_source.iter().map(|vr|{
        let var_name = vr.name;
        let no_source_backtrace = find_enum_variant_attr(vr.variant, "no_source_backtrace").is_some();

        if no_source_backtrace {
            quote!(  ErrorSource:: #var_name (_)  => { BacktraceInfo::empty() }     )
        } else {
            quote!(  ErrorSource:: #var_name (src)  => { src.provide_backtrace() }  )
        }
    }).collect::<Vec<_>>();

    let err_src_debug_impl_match_branches: Vec<proc_macro2::TokenStream> = enum_variants_wo_no_source.iter().map(|vr|{
        let var_name = vr.name;
        quote! (
            #var_name(ref src)  => { write!(f, "{}", src) }
        )
    }).collect::<Vec<_>>();

    let err_src_impl = quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl crate::util::backtrace::BacktraceCopyProvider for ErrorSource {
            fn provide_backtrace(&self) -> crate::util::BacktraceInfo {
                use crate::util::BacktraceInfo;
                match self {
                    ErrorSource::NoSource => { BacktraceInfo::empty() }
                    #(#err_src_bt_provider_impl_match_branches)*
                    // ErrorSource::ParseBigDecimalError(_)  => { BacktraceInfo::empty()  }
                    // ErrorSource::CurrencyFormatError(src) => { src.provide_backtrace() }
                }
            }
        }

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl core::fmt::Debug for ErrorSource {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                use ErrorSource::*;
                match self {
                    NoSource => { write!(f, "No source") }
                    #(#err_src_debug_impl_match_branches)*
                }
            }
        }
    };

    let from_impl: Vec<proc_macro2::TokenStream> = enum_variants_wo_no_source.iter().map(|vr|{
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

        quote! (
            // impl From<CurrencyFormatError> for ParseAmountError {
            //     fn from(error: CurrencyFormatError) -> Self { ParseAmountError::with_from(ErrorKind::IncorrectCurrency, error) }
            // }

            impl From< #from_err_type_name > for #err_struct_name {
                fn from(error: #from_err_type_name ) -> Self { #err_struct_name::with_from(ErrorKind:: #err_struct_kind_name, error) }
            }
        )
    }).collect::<Vec<_>>();

    let into_impl: Vec<proc_macro2::TokenStream> = enum_variants_wo_no_source.iter().map(|ref el| {
        let var_name: &syn::Ident = el.name;
        let var_arg_type: &syn::Type = el.first_arg_type
            .expect(&format!("first_arg_type is expected for {}.", var_name));

        quote! (
            // impl Into<ErrorSource> for CurrencyFormatError {
            //     fn into(self) -> ErrorSource { ErrorSource::CurrencyFormatError22(self) }
            // }

            #[allow(unused_imports)]
            #[allow(unused_qualifications)]
            impl Into<ErrorSource> for #var_arg_type {
                fn into(self) -> ErrorSource { ErrorSource:: #var_name (self) }
            }
        )
    }).collect::<Vec<_>>();


    let err_impl_ts: proc_macro::TokenStream = err_src_impl.into();

    // T O D O: probably it can be concatenated in standard way.
    let mut all = proc_macro::TokenStream::new();
    all.add_ts(err_impl_ts);
    all.add_pm2_tss(into_impl);
    all.add_pm2_tss(from_impl);
    all
}


#[derive(Debug)]
struct ErrorSourceEnumVariant<'a> {
    variant: & 'a syn::Variant,
    name: & 'a syn::Ident,
    first_arg_type: Option<& 'a syn::Type>,
}
#[derive(Debug)]
struct ErrorSourceEnum<'a> {
    #[allow(dead_code)]
    name: & 'a syn::Ident,
    variants: Vec<ErrorSourceEnumVariant<'a>>,
}

fn get_error_source_enum_variants<'a>(ast: & 'a syn::DeriveInput) -> ErrorSourceEnum<'a> {
    let enum_name: &syn::Ident = &ast.ident;

    let mut variants: Vec<ErrorSourceEnumVariant<'a>> = Vec::new();

    if let syn::Data::Enum(ref data_enum) = ast.data {

        data_enum.variants.iter().for_each(|vr| {
            let enum_el: &syn::Variant = vr;
            let variant_name: &syn::Ident = &vr.ident;

            if let syn::Fields::Unnamed(ref fields) = vr.fields {
                fields.unnamed.iter().for_each(|f| {
                    variants.push(ErrorSourceEnumVariant {
                        variant: enum_el,
                        name: variant_name,
                        first_arg_type: Some(&f.ty),
                    });
                });
            };

            if let syn::Fields::Unit = vr.fields {
                assert_eq!(variant_name, "NoSource",
                    "Unexpected enum variant Unit in enum {}.{} (only 'NoSource' Unit variant is expected).", enum_name, variant_name);
                variants.push(ErrorSourceEnumVariant { variant: enum_el, name: variant_name, first_arg_type: None });
            };
        })
    }
    else {
        panic!("Type {} should be enum", enum_name);
    }

    ErrorSourceEnum {
        name: enum_name,
        variants,
    }
}


fn find_attr<'a>(attrs: & 'a Vec<syn::Attribute>, attr_name: &str) -> Option<& 'a syn::Attribute> {
    attrs.iter().find(|attr|{
        let segments = &attr.meta.path().segments;
        let attr_name_as_path = segments.iter().map(|s| s.ident.to_string() ).collect::<String>();
        attr_name_as_path == attr_name
    })
}

#[allow(dead_code)]
fn attr_list_as_pm2_token_tree_vector(attr: &syn::Attribute) -> Option<Vec<proc_macro2::TokenTree>> {
    match &attr.meta {
        syn::Meta::List(ref meta_list) => {
            use quote::ToTokens;

            let tokens: &proc_macro2::TokenStream = &meta_list.tokens;
            let as_token_tee_2_vector: Vec<proc_macro2::TokenTree> = tokens.to_token_stream().into_iter().map(|t|{t}).collect::<Vec<_>>();
            Some(as_token_tee_2_vector)
        }
        _ => None
    }
}

fn attr_list_as_string(attr: &syn::Attribute) -> Option<String> {
    match &attr.meta {
        syn::Meta::List(ref meta_list) => {
            use quote::ToTokens;

            let tokens: &proc_macro2::TokenStream = &meta_list.tokens;
            let as_token_tee_2_vector: String = tokens.to_token_stream().into_iter().map(|t|{t.to_string()}).collect::<String>();
            Some(as_token_tee_2_vector)
        }
        _ => None
    }
}

fn find_enum_variant_attr<'a>(variant: & 'a syn::Variant, attr_name: & str) -> Option<& 'a syn::Attribute> {
    find_attr(&variant.attrs, attr_name)
}


trait AddPMTokenStream {
    fn add_ts(& mut self, other_ts: proc_macro::TokenStream);
}
trait AddPM2TokenStream {
    fn add_pm2_ts(& mut self, other_ts: proc_macro2::TokenStream);
}
trait AddPM2TokenStreams {
    fn add_pm2_tss(& mut self, other_ts: Vec<proc_macro2::TokenStream>);
}

impl AddPMTokenStream for proc_macro::TokenStream {
    fn add_ts(& mut self, ts: proc_macro::TokenStream) {
        self.extend(ts.into_iter());
    }
}
impl AddPM2TokenStream for proc_macro::TokenStream {
    fn add_pm2_ts(&mut self, ts: proc_macro2::TokenStream) {
        use quote::ToTokens;
        let as_ts: proc_macro::TokenStream = ts.to_token_stream().into();
        self.extend(as_ts);
    }
}
impl AddPM2TokenStreams for proc_macro::TokenStream {
    fn add_pm2_tss(&mut self, ts: Vec<proc_macro2::TokenStream>) {
        ts.iter().for_each(|ts_part|{
            use quote::ToTokens;
            let as_ts: proc_macro::TokenStream = ts_part.to_token_stream().into();
            self.extend(as_ts);
        });
    }
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
