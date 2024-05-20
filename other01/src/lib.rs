



// pub struct Other01Struct {}


/*
pub fn add456(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add456(2, 2);
        assert_eq!(result, 4);
    }
}
*/



extern crate proc_macro;
extern crate syn;
extern crate quote;

use proc_macro::TokenStream;

#[proc_macro_derive(TlbormDerive22, attributes(tlborm_helper22))]
pub fn tlborm_derive22(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}


#[proc_macro_derive(TlbormDerive, attributes(tlborm_helper))]
pub fn tlborm_derive(_item: TokenStream) -> TokenStream {
    TokenStream::new()
}

/*
#[proc_macro_derive(MyStaticTypedErrorDerive, attributes(my_static_types_error_attr_helper))]
pub fn my_static_types_error(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = impl_hello_world(&ast);

    // Return the generated impl
    gen.parse().unwrap()
}


fn impl_hello_world(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    quote! {
        impl HelloWorld for #name {
            fn hello_world() {
                println!("Hello, World! My name is {}", stringify!(#name));
            }
        }
    }
}
*/


// use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use quote::quote_spanned;
use syn::{Attribute, DeriveInput, Ident, Type, TypePath, Variant};
// use syn::token::Paren;
// use syn;

#[proc_macro_derive(HelloMacro)]
pub fn hello_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    let ast = syn::parse(input).unwrap();

    // Build the trait implementation
    impl_hello_macro(&ast)
}

fn impl_hello_macro(ast: &syn::DeriveInput) -> proc_macro::TokenStream {
    let name = &ast.ident;
    let gen = quote! {
        impl HelloMacro for #name {
            fn hello_macro() {
                println!("Hello, Macro! My name is {}!", stringify!(#name));
            }
        }
    };
    gen.into()
}


#[proc_macro_derive(MyStaticStructError, attributes(StaticStructErrorType))]
pub fn my_static_struct_error_macro_derive(input: TokenStream) -> TokenStream {
    // Construct a representation of Rust code as a syntax tree
    // that we can manipulate
    // let ast: proc_macro2::TokenStream = syn::parse(input).unwrap();
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let attrs33: &Vec<syn::Attribute> = &ast.attrs;
    match attrs33.first() {
        None => {}
        Some(ref attr) => {
            panic!("attr: {:?}", attr);
        }
    }
    // attrs.

    //
    // let power = attributes_search(&ast.attrs, "power").expect("missing power attribute");

    // Build the trait implementation
    impl_my_static_struct_error(&ast)
}

fn impl_my_static_struct_error(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    // TODO: how to write it short
    let source_field_exists: bool = if let syn::Data::Struct(ref data) = ast.data {
        if let syn::Fields::Named(ref fields) = data.fields {
            fields.named.iter().any(|el| el.ident
                .as_ref()
                .map(|ident_name| ident_name.to_string() == "source")
                .unwrap_or(false)
            )
        }
        else { false }
    } else { false };

    /*
    if let syn::Data::Enum(ref data) = ast.data {
        // panic!("Struct is not supported");
    }
    */

    let err_impl_without_source = quote! {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl #name {
            // It can be generated by macro
            pub fn new(kind: ErrorKind) -> Self {
                use crate::util::backtrace::NewBacktracePolicy;
                use crate::util::BacktraceInfo;
                Self { kind, backtrace: BacktraceInfo::new() }
            }
            // It can be generated by macro
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
            // TODO: generate it when `From` is generated
            // It can be generated by macro
            // pub fn with_from<ES: Into<ErrorSource>>(kind: ErrorKind, source: ES) -> Self {
            //     use crate::util::backtrace::NewBacktracePolicy;
            //     use crate::util::BacktraceInfo;
            //     let src = source.into();
            //     Self { kind, backtrace: BacktraceInfo::inherit_from(&src), source: src }
            // }
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

    let err_impl = if source_field_exists { err_impl_with_source } else { err_impl_without_source };
    let err_impl_ts: TokenStream = err_impl.into();
    let backtrace_provider_ts: TokenStream = err_backtrace_provider_impl.into();
    let err_display_ts: TokenStream = err_display_impl.into();

    // TODO: probably it can be written a bit easier
    let mut all = TokenStream::new(); // proc_macro::TokenStream

    all.extend(err_impl_ts.into_iter());
    all.extend(backtrace_provider_ts.into_iter());
    all.extend(err_display_ts.into_iter());

    all
}



#[proc_macro_derive(MyStaticStructErrorSource, attributes(struct_error_type, from_error_kind))]
pub fn my_static_struct_error_source_macro_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();
    impl_my_static_struct_error_source(&ast)
}

// #[derive(MyStaticStructErrorSource)]
// struct Struct {
//     #[StaticStructErrorType] field: ()
// }
// #[derive(HelperAttr)]
// struct Struct {
//     #[helper] field: ()
// }

fn impl_my_static_struct_error_source(ast: &syn::DeriveInput) -> TokenStream {
    let name = ast.ident.to_string();

    let attrs33: &Vec<syn::Attribute> = &ast.attrs;
    match attrs33.first() {
        None => {}
        Some(ref attr) => {
            // panic!("attr 4545: {:?}", attr);
        }
    }

    /*
    let name = &ast.ident;

    let mut i = 0;

    if let syn::Data::Enum(ref data_enum) = ast.data {
        // if let syn::DataEnum::Named(ref fields) = data.enum_token {
        //     ;
        // }

        // let enum_idents = data_enum.variants.iter().map(|v|v.as_ref()).map(|var| var.ident);
        // let enum_idents = data_enum.variants.iter().map(|el| el.ident.as_ref().ident);

        let enum_idents: Vec<String> = data_enum.variants.iter().map(|el|
            el.ident.to_string()
        ).collect::<Vec<String>>();
        let enum_idents: Vec<String> = data_enum.variants.iter().map(|el| {

            let ident_name = el.ident.to_string();
            // if i == 1 { panic!("### {:?}", el); }

            if let syn::Fields::Named(ref fields) = el.fields {
                // panic!("### Named fields");
                fields.named.iter().map(|el| el.ident
                    .as_ref()
                    .map(|ident_name| {
                        panic!("named ident_name.to_string(): {:?}", ident_name);
                    })
                    .unwrap_or(false)
                ).collect::<Vec<_>>();
            };

            if let syn::Fields::Unnamed(ref fields) = el.fields {
                // panic!("### Unnamed 01 {}", ident_name);
                let _aa = fields.unnamed.iter().map(|el|{

                    // panic!("### Unnamed 02 {}", ident_name);
                    // panic!("### Unnamed 02 {} {}", ident_name, el.ident.to_string());
                    // panic!("### Unnamed 02 {} {}", ident_name, el.ident.is_none());

                    // panic!("### Unnamed 02 {} attrs {:?}", ident_name, el.attrs);
                    // panic!("### Unnamed 02 {} colon_token {:?}", ident_name, el.colon_token);
                    // panic!("### Unnamed 02 {} ty {:?}", ident_name, el.ty);
                    // panic!("### Unnamed 02 {} vis {:?}", ident_name, el.vis);
                    if let Type::Path(ref type_path) = el.ty {
                        // panic!("### Unnamed 02 {} ty44334 {:?}", ident_name, type_path.path.segments);
                        // type_path.path.segments
                        // let type1st = type_path.path.segments.first().map(|ee| ee.ident.to_string() ).unwrap();
                        // let type1st = type_path.path.segments.iter().map(|ee| ee.ident.to_string() ).collect::<String>();
                        // unstable api
                        // let type1st = type_path.path.segments.iter().map(|ee| ee.ident.to_string() ).intersperse(", ".to_string()).collect::<String>();
                        // let type1st = type_path.path.segments.iter().map(|ee| ee.ident.to_string() ).collect::<Vec<String>>().join("::");

                        let enum_variant_name = &ident_name;
                        let enum_variant_arg_type = type_path.path.segments.iter().map(|ee| ee.ident.to_string() ).collect::<Vec<String>>().join("::");;

                        panic!("### Unnamed 03 {} ty7767 {:?}", ident_name, enum_variant_arg_type);
                    }

                    match el.ident {
                        None => {}
                        Some(ref el_ident) => {
                            panic!("### Unnamed 02 {} {}", ident_name, el_ident.to_string());
                        }
                    }

                    el.ident
                    .as_ref()
                    .map(|unnamed_ident_name| {
                        panic!("### unnamed of {:?}: {:?}", ident_name, unnamed_ident_name);
                    })
                    .unwrap_or(false)}
                ).collect::<Vec<_>>();

                let _bb: syn::token::Paren = fields.paren_token;
                //panic!("{}", _bb.to_string())
            };

            if let syn::Fields::Unit = el.fields {
                // panic!("### Unit {}", ident_name);
            };

            // "fdfdf".to_string()
            i += 1;
            ident_name
        }).collect::<Vec<String>>();

        // panic!("enum_idents: {:?}", enum_idents);

        // let enum_idents = data_enum.variants.iter().map(|&el| {
        //     match el {
        //         Variant { .. } => {}
        //     }
        // });
        //panic!("enum_idents: {:?}", enum_idents);

        data_enum.variants.iter().for_each(|var|
            // Variant {
            //   attrs: [
            //     Attribute { pound_token: Pound, style: AttrStyle::Outer, bracket_token: Bracket,
            //                 meta: Meta::List { path: Path { leading_colon: None, segments: [PathSegment { ident: Ident { ident: "error", span: #0 bytes(22738..22743) }, arguments: PathArguments::None }] },
            //     delimiter: MacroDelimiter::Paren(Paren),
            //     tokens: TokenStream [Literal { kind: Str, symbol: "No source", suffix: None, span: #0 bytes(22744..22755) }] } }],
            //     ident: Ident { ident: "NoSource", span: #0 bytes(22766..22774) },
            //     fields: Fields::Unit, discriminant: None }
            // compile_print!("### variant: {:?}", var)
            //panic!("### variant: {:?}", var.ident)

            // Variant { attrs: [Attribute {
            //    pound_token: Pound,
            //    style: AttrStyle::Outer,
            //    bracket_token: Bracket,
            //    meta: Meta::List { path: Path { leading_colon: None, segments: [PathSegment { ident: Ident { ident: "error", span: #0 bytes(22786..22791) },
            //    arguments: PathArguments::None }] },
            //    delimiter: MacroDelimiter::Paren(Paren),
            //    tokens: TokenStream [Literal { kind: Str, symbol: "Currency format error", suffix: None, span: #0 bytes(22792..22815) }] } }],
            //    ident: Ident { ident: "CurrencyFormatError", span: #0 bytes(22826..22845) },
            //
            //    fields: Fields::Unnamed {
            //      paren_token: Paren,
            //      unnamed: [Field {
            //        attrs: [],
            //        vis: Visibility::Inherited,
            //        mutability: FieldMutability::None,
            //        ident: None, colon_token: None, ty: Type::Path { qself: None, path: Path
            //        { leading_colon: None,
            //        segments: [PathSegment { ident: Ident { ident: "CurrencyFormatError", span: #0 bytes(22846..22865) },
            //        arguments: PathArguments::None }] } } }] }, discriminant: None }
            {}
        );
    }

    if let syn::Data::Struct(ref data) = ast.data {
        if let syn::Fields::Named(ref fields) = data.fields {

            // let field_vals = fields.named.iter().enumerate().map(|(i, field)| {
            //     let name = &field.ident;
            //     quote!(#name: row.try_get(#i)?)
            // });
            // let source_field_exists: bool = fields.named.iter().any(|el|{
            //     match el.ident {
            //         None => { false }
            //         Some(ref ident) => { ident.to_string() == "source" }
            //     }
            // });
            // let source_field_exists22: bool = fields.named.iter().any(|ref el|{el.ident.map(|ident|ident.to_string() == "source").unwrap_or(false)});
            // let source_field_exists22: bool = fields.named.iter().any(|el|{
            //     el.ident.as_ref().map(|ident|ident.to_string() == "source").unwrap_or(false)
            // });
            // panic!("source_field_exists: {source_field_exists}");

            // return TokenStream::from(quote!(
            // impl divedb_core::FromRow for #name {
            //     fn from_row(row: tokio_postgres::Row) -> Result<Self, anyhow::Error> {
            //         Ok(Self {
            //             #(#field_vals),*
            //         })
            //     }
            // }));
        }
    }

    if let syn::Data::Struct(ref data) = ast.data {
        if let syn::Fields::Named(ref fields) = data.fields {
            let field_vals = fields.named.iter().enumerate().map(|(i, field)| {
                // grab the name of the field
                let name = &field.ident;
                quote!(#name: row.try_get(#i)?)
            });
        }
    }
    */

    let enum_variants = get_error_source_enum_variants(ast);
    // panic!("### enum_variants: {:?}", enum_variants);
    let enum_variants_wo_no_source = enum_variants.variants.iter().filter(|el|{ el.name != "NoSource" });

    let enums_xxx: Vec<proc_macro2::TokenStream> = enum_variants_wo_no_source.clone().map(|el|{
        // let var_name = syn::Ident::new(&el.name, ast.ident.span()); // TODO: why do I use ast.ident.span() there ??? https://stackoverflow.com/questions/62370461/how-can-i-concatenate-a-string-to-an-ident-in-a-macro-derive
        let var_name = el.name; // TODO: why do I use ast.ident.span() there ??? https://stackoverflow.com/questions/62370461/how-can-i-concatenate-a-string-to-an-ident-in-a-macro-derive
        // TODO: use `src.provide_backtrace()`
        quote! (
            ErrorSource:: #var_name (_)  => { BacktraceInfo::empty() }
        )
    }).collect::<Vec<_>>();

    let enums_xxx22: Vec<proc_macro2::TokenStream> = enum_variants_wo_no_source.clone().map(|el|{
        // let var_name = syn::Ident::new(&el.name, ast.ident.span());
        let var_name = el.name;
        quote! (
            #var_name(ref src)  => { write!(f, "{:?}", src) }
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
                    #(#enums_xxx)*
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
                    NoSource                      => { write!(f, "No source") }
                    #(#enums_xxx22)*
                }
            }
        }
    };

    // let enums_xxx3: Vec<proc_macro2::TokenStream> = enum_variants_wo_no_source.clone().map(|el|{
    //     let err_struct_name = ; // syn::Ident::new(&el.name, ast.ident.span());
    //     let var_name = syn::Ident::new(&el.name, ast.ident.span());
    //     let var_name = syn::Ident::new(&el.name, ast.ident.span());
    //     quote! (
    //         impl From< #err_struct_name > for ParseAmountError {
    //             fn from(error: CurrencyFormatError) -> Self { ParseAmountError::with_from(ErrorKind::IncorrectCurrency, error) }
    //         }
    //     )
    // }).collect::<Vec<_>>();

    let into_impl: Vec<proc_macro2::TokenStream> = enum_variants_wo_no_source.clone().map(|ref el|{ // TODO: to avoid clone().
        // let err_struct_name = ; // syn::Ident::new(&el.name, ast.ident.span());
        //panic!("### 65656 01");
        // let var_name = syn::Ident::new(&el.name, ast.ident.span());
        let var_name: &Ident = el.name;
        // panic!("### 65656 02");

        // TODO: gather Type instead of manual recreating it !!!
        // let var_arg_type = syn::Type::Path(TypePath { qself: None, path: Path { leading_colon: None,  }  });// Ident::new(&el.first_arg_type.clone().unwrap(), ast.ident.span()); // TODO: remove clone
        // let var_arg_type: Option<&Type> = el.first_arg_type.as_ref();
        let var_arg_type: &Type = el.first_arg_type.unwrap();
        // panic!("### 65656 03");
        // panic!("### 65656 var_name: {:?}, var_arg_type: {:?}", var_name, var_arg_type);
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



    // quote! {
    //     fn aa() {}
    // }
    // .into()

    // err_src_impl.into()

    let err_impl_ts: TokenStream = err_src_impl.into();
    // let into_impl_ts: TokenStream = into_impl.into();

    // TODO: probably it can be written a bit easier
    let mut all = TokenStream::new(); // proc_macro::TokenStream

    all.extend(err_impl_ts.into_iter());
    // all.extend(into_impl_ts.into_iter());
    into_impl.iter().for_each(|rrr|{
        let as_ts: TokenStream = rrr.to_token_stream().into();
        all.extend(as_ts);
    });

    all
}


#[derive(Debug)]
struct ErrorSourceEnumVariant<'a> {
    name: & 'a Ident,
    first_arg_type: Option<& 'a Type>,
}
#[derive(Debug)]
struct ErrorSourceEnum<'a> {
    name: & 'a Ident,
    variants: Vec<ErrorSourceEnumVariant<'a>>,
}

fn get_error_source_enum_variants<'a>(ast: & 'a syn::DeriveInput) -> ErrorSourceEnum<'a> {
    let enum_name: &Ident = &ast.ident;

    let mut variants: Vec<ErrorSourceEnumVariant<'a>> = Vec::new();

    let mut i = 0;

    if let syn::Data::Enum(ref data_enum) = ast.data {

        // let enum_idents: Vec<String> = data_enum.variants.iter().map(|el|
        //     el.ident.to_string()
        // ).collect::<Vec<String>>();

        data_enum.variants.iter().for_each(|el| {
            let variant_name: &Ident = &el.ident;

            if let syn::Fields::Named(ref fields) = el.fields {
                fields.named.iter().map(|el| el.ident
                    .as_ref()
                    .map(|ident_name| {
                        panic!("named ident_name.to_string(): {:?}", ident_name);
                    })
                    .unwrap_or(false)
                ).collect::<Vec<_>>();
            };

            if let syn::Fields::Unnamed(ref fields) = el.fields {

                let _aa = fields.unnamed.iter().for_each(|el| {

                    variants.push(ErrorSourceEnumVariant { name: variant_name, first_arg_type: Some(&el.ty) });

                    // if let Type::Path(ref type_path) = el.ty {
                    //     let enum_variant_name = &variant_name;
                    //     //let enum_variant_arg_type: String = type_path.path.segments.iter().map(|ee| ee.ident.to_string()).collect::<Vec<String>>().join("::");
                    //     let enum_variant_arg_type: String = type_path.path.segments.iter().map(|ee| ee.ident.to_string()).collect::<Vec<String>>().join("::");
                    //
                    //     variants.push(ErrorSourceEnumVariant<'a> { name: enum_variant_name, first_arg_type: Some(enum_variant_arg_type) });
                    // }

                    // el.ident
                    //     .as_ref()
                    //     .map(|unnamed_ident_name| {
                    //         panic!("### unnamed of {:?}: {:?}", variant_name, unnamed_ident_name);
                    //     })
                    //     .unwrap_or(false)
                });
            };

            if let syn::Fields::Unit = el.fields {
                if variant_name != "NoSource" {
                    panic!("### Unexpected enum variant Unit in enum {}.{} (only 'NoSource' Unit variant is expected).", enum_name, variant_name);
                }
                variants.push(ErrorSourceEnumVariant { name: variant_name, first_arg_type: None });
            };
        })
    }
    else {
        panic!("Type {:?} should be enum", enum_name);
    }

    ErrorSourceEnum {
        name: enum_name,
        variants,
    } //<'a>
}

/*
#[derive(Debug)]
struct ErrorSourceEnumVariant {
    name: String,
    first_arg_type: Option<String>,
}
#[derive(Debug)]
struct ErrorSourceEnum {
    name: String,
    variants: Vec<ErrorSourceEnumVariant>,
}

fn get_error_source_enum_variants(ast: &syn::DeriveInput) -> ErrorSourceEnum {
    let enum_name: String = ast.ident.to_string();

    let mut variants: Vec<ErrorSourceEnumVariant> = Vec::new();

    let mut i = 0;

    if let syn::Data::Enum(ref data_enum) = ast.data {

        // let enum_idents: Vec<String> = data_enum.variants.iter().map(|el|
        //     el.ident.to_string()
        // ).collect::<Vec<String>>();

        data_enum.variants.iter().for_each(|el| {
            let variant_name = el.ident.to_string();

            if let syn::Fields::Named(ref fields) = el.fields {
                fields.named.iter().map(|el| el.ident
                    .as_ref()
                    .map(|ident_name| {
                        panic!("named ident_name.to_string(): {:?}", ident_name);
                    })
                    .unwrap_or(false)
                ).collect::<Vec<_>>();
            };

            if let syn::Fields::Unnamed(ref fields) = el.fields {

                let _aa = fields.unnamed.iter().for_each(|el| {

                    if let Type::Path(ref type_path) = el.ty {
                        let enum_variant_name = &variant_name;
                        let enum_variant_arg_type: String = type_path.path.segments.iter().map(|ee| ee.ident.to_string()).collect::<Vec<String>>().join("::");

                        variants.push(ErrorSourceEnumVariant { name: enum_variant_name.to_string(), first_arg_type: Some(enum_variant_arg_type) });
                    }

                    // el.ident
                    //     .as_ref()
                    //     .map(|unnamed_ident_name| {
                    //         panic!("### unnamed of {:?}: {:?}", variant_name, unnamed_ident_name);
                    //     })
                    //     .unwrap_or(false)
                });
            };

            if let syn::Fields::Unit = el.fields {
                if variant_name != "NoSource" {
                    panic!("### Unexpected enum variant Unit in enum {}.{} (only 'NoSource' Unit variant is expected).", enum_name, variant_name);
                }
                variants.push(ErrorSourceEnumVariant { name: variant_name, first_arg_type: None });
            };
        })
    }
    else {
        panic!("Type {:?} should be enum", enum_name);
    }

    ErrorSourceEnum {
        name: enum_name,
        variants,
    }
}
*/


// -------------------------------------------------------------------------------------------------
//                                        Private tests
// -------------------------------------------------------------------------------------------------


// Tests for private methods/behavior
// Other test are located in ${project}/tests/currency_test.rs
//
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_of_error_source() {
    }
}
