

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


pub fn type_path_to_string(path: &syn::TypePath) -> String {
    use quote::ToTokens;
    path.to_token_stream().to_string()
    // path.path.segments.iter().map(|s| s.ident.to_string() ).collect::<String>()
}
#[allow(dead_code)]
pub fn type_path_to_string_without_spaces(path: &syn::TypePath) -> String {
    remove_spaces_from_type_string(&type_path_to_string(path))
}
fn remove_spaces_from_type_string(type_as_str: &String) -> String {

    let mut trimmed = String::new();

    let mut prev_non_space_is_alpha = false;
    let mut prev_is_space = false;

    type_as_str.chars().for_each(|ch|{

        let is_space = ch.is_ascii_whitespace();

        if is_space {
            prev_is_space = true;
        }
        else {
            let is_alpha = ch.is_alphabetic();

            if is_alpha && prev_is_space && prev_non_space_is_alpha {
                trimmed.push(' ');
            }
            trimmed.push(ch);

            prev_is_space = false;
            prev_non_space_is_alpha = is_alpha;
        }
    });

    trimmed
}

fn remove_space_chars_impl(str: &str) -> String {
    let mut res = String::with_capacity(str.len());
    str.chars().for_each(|ch|{
        if !ch.is_ascii_whitespace() {
            res.push(ch);
        }
    });
    res
}

#[allow(dead_code)]
pub trait StringOp {
    fn remove_space_chars(&self) -> String;
}
impl StringOp for String {
    fn remove_space_chars(&self) -> String { remove_space_chars_impl(self.as_str()) }
}

pub trait OptionStringOp {
    fn is_eq_to_str(&self, str: &str) -> bool;
}
impl OptionStringOp for Option<String> {
    fn is_eq_to_str(&self, str: &str) -> bool {
        // It moves ??? Unexpected a bit???
        // self.is_some_and(|t| t.as_str() == str)
        match self {
            None => { false }
            Some(ref self_string) => { str == self_string.as_str() }
        }
    }
}
impl OptionStringOp for Option<&str> {
    fn is_eq_to_str(&self, str: &str) -> bool {
        // It moves ??? Unexpected a bit???
        // self.is_some_and(|t| t.as_str() == str)
        match self {
            None => { false }
            Some(ref self_string) => { str == *self_string }
        }
    }
}

pub fn type_to_string(t: &syn::Type) -> String {
    use syn::Type;
    match t {
        Type::Array(_)       => { unimplemented!() }
        Type::BareFn(_)      => { unimplemented!()}
        Type::Group(_)       => { unimplemented!() }
        Type::ImplTrait(_)   => { unimplemented!() }
        Type::Infer(_)       => { unimplemented!() }
        Type::Macro(_)       => { unimplemented!() }
        Type::Never(_)       => { unimplemented!() }
        Type::Paren(_)       => { unimplemented!() }
        Type::Path(path)     => { type_path_to_string(&path) }
        Type::Ptr(_)         => { unimplemented!() }
        Type::Reference(_)   => { unimplemented!() }
        Type::Slice(_)       => { unimplemented!() }
        Type::TraitObject(_) => { unimplemented!() }
        Type::Tuple(_)       => { unimplemented!() }
        Type::Verbatim(_)    => { unimplemented!() }
        _                    => { unimplemented!() }
    }
}

pub fn type_to_string_without_spaces(t: &syn::Type) -> String {
    remove_spaces_from_type_string(&type_to_string(t))
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


#[cfg(test)]
mod tests {
    use super::*;

    fn remove_spaces_from_type_str(str_type: &str) -> String {
        remove_spaces_from_type_string(&str_type.to_string())
    }

    #[test]
    fn test_remove_spaces_from_type() {
        assert_eq!(remove_spaces_from_type_str("anyhow::Error"), "anyhow::Error");
        assert_eq!(remove_spaces_from_type_str("anyhow :: Error"), "anyhow::Error");
        assert_eq!(remove_spaces_from_type_str(" anyhow :: Error"), "anyhow::Error");
        assert_eq!(remove_spaces_from_type_str("  anyhow  ::  Error  "), "anyhow::Error");

        assert_eq!(remove_spaces_from_type_str("Box<dyn std::error::Error>"), "Box<dyn std::error::Error>");
        assert_eq!(remove_spaces_from_type_str(" Box < dyn std :: error :: Error > "), "Box<dyn std::error::Error>");
        assert_eq!(remove_spaces_from_type_str("  Box  <  dyn  std  ::  error  ::  Error  >  "), "Box<dyn std::error::Error>");
    }
}
