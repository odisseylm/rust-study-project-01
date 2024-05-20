


pub fn find_attr<'a>(attrs: & 'a Vec<syn::Attribute>, attr_name: &str) -> Option<& 'a syn::Attribute> {
    attrs.iter().find(|attr|{
        let segments = &attr.meta.path().segments;
        let attr_name_as_path = segments.iter().map(|s| s.ident.to_string() ).collect::<String>();
        attr_name_as_path == attr_name
    })
}


#[allow(dead_code)]
pub fn attr_list_as_pm2_token_tree_vector(attr: &syn::Attribute) -> Option<Vec<proc_macro2::TokenTree>> {
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


pub fn attr_list_as_string(attr: &syn::Attribute) -> Option<String> {
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


pub fn find_enum_variant_attr<'a>(variant: & 'a syn::Variant, attr_name: & str) -> Option<& 'a syn::Attribute> {
    find_attr(&variant.attrs, attr_name)
}


pub trait AddPMTokenStream {
    fn add_ts(& mut self, other_ts: proc_macro::TokenStream);
}
pub trait AddPM2TokenStream {
    fn add_pm2_ts(& mut self, other_ts: proc_macro2::TokenStream);
}
pub trait AddPM2TokenStreams {
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
