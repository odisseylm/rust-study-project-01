
// #[macro_use]
// mod compile_log_macros;
// include!("./compile_log_macros.rs");

use quote::{ quote, /*TokenStreamExt, ToTokens*/ };
use crate::util::make_ident;


pub(crate) fn generate_assert_tuple_len_is_impl(max_tuple_len: usize) -> proc_macro2::TokenStream {

    let assert_tuple_len_is_functions = (1..max_tuple_len)
        .into_iter()
        .map(|i|{
            let method_ident = make_ident(format!("assert_tuple_len_is_{i}"));
            let types = types_list(i);
            quote! {
                pub fn #method_ident < #(#types),* >(_tuple: &(#(#types),*)) {}
            }
        })
        .collect::<Vec<_>>();

    let q = quote! {
        pub fn assert_tuple_len_is_0(_tuple: &()) {}
        #(#assert_tuple_len_is_functions)*
    };
    q.into()
}


pub(crate) fn generate_all_tuple_access(max_tuple_len: usize) -> proc_macro::TokenStream {

    let trait_def = generate_tuple_access_trait_impl(max_tuple_len);

    let impl_for_0 = generate_tuple_0_access_impl(max_tuple_len);

    let impls =  (1..max_tuple_len)
        .into_iter()
        .map(|tuple_len| generate_tuple_access_impl(max_tuple_len, tuple_len))
        .collect::<Vec<_>>();

    // let assert_len_functions = generate_assert_tuple_len_is_impl(max_tuple_len);

    let out_ps2: proc_macro2::TokenStream = quote! {
        #trait_def
        #impl_for_0
        #(#impls)*

        // #assert_len_functions
    };
    out_ps2.into()
}



/**
Generates code like
```
pub trait TupleAccess {
    type Elem0;
    fn _0(&self) -> Option<&Self::Elem0>;
    type Elem1;
    fn _1(&self) -> Option<&Self::Elem1>;
    type Elem2;
    fn _2(&self) -> Option<&Self::Elem2>;
    type Elem3;
    fn _3(&self) -> Option<&Self::Elem3>;
    type Elem4;
    fn _4(&self) -> Option<&Self::Elem4>;
}
```
*/
pub(crate) fn generate_tuple_access_trait_impl(max_tuple_len: usize) -> proc_macro2::TokenStream {
    use proc_macro2::TokenStream as PM2TS;

    let rows: Vec<PM2TS> = (0..max_tuple_len)
        .into_iter()
        .map(|i|{
            let elem_type_ident = make_ident(format!("Elem{i}"));
            let method_ident = make_ident(format!("_{i}"));

            quote! {
                type #elem_type_ident;
                fn #method_ident(&self) -> Option<&Self:: #elem_type_ident>;
            }
        })
        .collect();

    let out: PM2TS = quote!{
        pub trait TupleAccess {
            #(#rows)*
        }
    };
    out.into()
}




/**
Generates code like
```no_build
impl <T0,T1> TupleAccess for (T0,T1) {
    type Elem0 = T0;
    #[inline(always)]
    fn _0(&self) -> Option<&Self::Elem0> { Some(&self.0) }

    type Elem1 = T1;
    #[inline(always)]
    fn _1(&self) -> Option<&Self::Elem1> { Some(&self.1) }

    // Not supported
    type Elem2 = T0;
    #[inline(always)]
    fn _2(&self) -> Option<&Self::Elem02> { None }
}
```
*/
fn generate_tuple_access_impl(max_tuple_len: usize, current_tuple_len: usize)
                           -> proc_macro2::TokenStream {
    use proc_macro2 as pm2;

    let types = types_list(current_tuple_len);

    let matched_type_rows: Vec<pm2::TokenStream> = (0..current_tuple_len)
        .into_iter()
        .map(|i| {
            let index = syn::Index::from(i);
            let gen_elem_type_ident = make_ident(format!("T{i}"));
            let elem_type_ident = make_ident(format!("Elem{i}"));
            let method_ident = make_ident(format!("_{i}"));

            quote! {
                type #elem_type_ident = #gen_elem_type_ident;
                #[inline(always)]
                fn #method_ident(&self) -> Option<&Self:: #elem_type_ident> { Some(&self. #index) }
            }
        })
        .collect::<Vec<_>>();


    let unmatched_type_rows =  (current_tuple_len..max_tuple_len)
        .into_iter()
        .map(|i| {
            let elem_type_ident = make_ident(format!("Elem{i}"));
            let method_ident = make_ident(format!("_{i}"));

            quote! {
                type #elem_type_ident = T0;
                #[inline(always)]
                fn #method_ident(&self) -> Option<&Self:: #elem_type_ident> { None }
            }
        })
        .collect::<Vec<_>>();


    let out: pm2::TokenStream = quote! {
        impl < #(#types),* > TupleAccess for ( #(#types),* ,) {
            #(#matched_type_rows)*
            #(#unmatched_type_rows)*
        }
    };
    out.into()
}



fn generate_tuple_0_access_impl(max_tuple_len: usize)
                           -> proc_macro2::TokenStream {
    use proc_macro2 as pm2;
    use proc_macro2::TokenStream as PM2TS;

    let unmatched_type_rows: Vec<pm2::TokenStream> = (0..max_tuple_len)
        .into_iter()
        .map(|i| {
            let elem_type_ident = make_ident(format!("Elem{i}"));
            let method_ident = make_ident(format!("_{i}"));

            quote! {
                type #elem_type_ident = ();
                #[inline(always)]
                fn #method_ident(&self) -> Option<&Self:: #elem_type_ident> { None }
            }
        })
        .collect::<Vec<_>>();

    let out: PM2TS = quote! {
        impl TupleAccess for () {
            #(#unmatched_type_rows)*
        }
    };
    out.into()
}


pub(crate) fn generate_all_tuple_len_traits(max_tuple_len: usize) -> proc_macro::TokenStream {

    let trait_def = generate_tuple_len_trait();

    let impl_for_0 = generate_tuple_0_len_impl();

    let impls =  (1..max_tuple_len)
        .into_iter()
        .map(|tuple_len| generate_tuple_len_impl(tuple_len))
        .collect::<Vec<_>>();

    let out_ps2: proc_macro2::TokenStream = quote! {
        #trait_def
        #impl_for_0
        #(#impls)*
    };
    out_ps2.into()
}



/**
Generates code like
```
pub trait TupleLen {
    const LENGTH: usize;
    fn tuple_len(&self) -> usize { Self::LENGTH }
    // ?? Can we safely use such short name ??
    fn len(&self) -> usize { Self::LENGTH }
}
```
*/
pub(crate) fn generate_tuple_len_trait() -> proc_macro2::TokenStream {
    let out: proc_macro2::TokenStream = quote!{
        pub trait TupleLen {
            const LENGTH: usize;
            fn tuple_len(&self) -> usize { Self::LENGTH }
            fn len(&self) -> usize { Self::LENGTH }
        }
    };
    out.into()
}

/**
Generates code like
``
impl <T0,T1> TupleLen for (T0,T1) {
    const LENGTH: usize = 2;
    #[inline(always)]
    fn tuple_len(&self) -> usize { 2 }
    #[inline(always)]
    fn len(&self) -> usize { 2 }
}
``
*/
fn generate_tuple_len_impl(tuple_len: usize)
                           -> proc_macro2::TokenStream {
    use proc_macro2 as pm2;
    use proc_macro2::TokenStream as PM2TS;

    let current_tuple_len_literal = pm2::TokenTree::Literal(
        pm2::Literal::usize_unsuffixed(tuple_len));

    let types = types_list(tuple_len);

    let out: PM2TS = quote! {
        impl < #(#types),* > TupleLen for ( #(#types),* ,) {
            const LENGTH: usize = #current_tuple_len_literal;
            #[inline(always)]
            fn tuple_len(&self) -> usize { #current_tuple_len_literal }
            #[inline(always)]
            fn len(&self) -> usize { #current_tuple_len_literal }
        }
    };
    out.into()
}

fn generate_tuple_0_len_impl() -> proc_macro2::TokenStream {
    let out: proc_macro2::TokenStream = quote! {
        impl TupleLen for () {
            const LENGTH: usize = 0;
            #[inline(always)]
            fn tuple_len(&self) -> usize { 0 }
            #[inline(always)]
            fn len(&self) -> usize { 0 }
        }
    };
    out.into()
}


/**
 * Generates types quote like 'T0,T1,T2...'
 */
fn types_list(type_count: usize) -> Vec<proc_macro2::TokenStream> {
    (0..type_count)
        .into_iter()
        .map(|i| make_ident(format!("T{i}")))
        .collect::<Vec<_>>()
}
