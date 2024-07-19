use itertools::Position;
use quote::ToTokens;



pub(crate) fn make_ident(ident: String) -> proc_macro2::TokenStream {
    // proc_macro2::TokenTree::Ident(proc_macro2::Ident::new(ident.as_str(), quote!{}.span()))

    // or
    let ident: syn::Ident = syn::parse_str(ident.as_str())
        .expect(&format!("Error of converting [\"{ident}\"] to Ident."));
    ident.into_token_stream()
}


pub(crate) fn split_params(params: proc_macro::TokenStream) -> Vec<proc_macro2::TokenStream> {
    use proc_macro2::TokenTree;
    let params2: proc_macro2::TokenStream = params.into();

    let mut current_param_as_stream = proc_macro2::TokenStream::new();
    let mut params_vec = Vec::<proc_macro2::TokenStream>::new();

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

    params_vec
}
