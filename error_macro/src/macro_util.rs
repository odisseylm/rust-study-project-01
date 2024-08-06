use core::str::FromStr;
use std::collections::{HashMap, HashSet};
use strum_macros::Display;


// A bit hacky, but working solution.
// #[macro_use]
// #[path = "./compile_log_macros.rs"]
// mod compile_log_macros; // to import compile_log_warn

// Another a bit hacky, but working solution.
include!("./compile_log_macros.rs");

// #[macro_use(compile_log_warn)]
// use crate::compile_log_macros;

// // Does not work!
// extern crate self as xxx;
// use xxx::compile_log_macros::compile_log_warn;

// Does not work!
// use crate::compile_log_macros::compile_log_warn;

// Does not work!
// #[macro_use(compile_log_warn)]
// use crate::compile_log_macros;

// Does not work!
// #[macro_use(compile_log_warn)]
// extern crate self as xxx;
// use xxx::compile_log_macros::compile_log_warn;


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
            let as_token_tee_2_vector: String = tokens.to_token_stream().into_iter()
                .map(|t|t.to_string()).collect::<String>();
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
    let mut is_prev_non_space_char_alpha_or_digit = false;
    let mut is_prev_space = false;
    str.chars().for_each(|ch|{

        if ch.is_ascii_whitespace() {
            // we will add ' ' or not add later depending on prev and next non-space chars
        }
        else if ch.is_ascii_alphanumeric() {
            if is_prev_space && is_prev_non_space_char_alpha_or_digit {
                res.push(' ');
            }
            res.push(ch);
            is_prev_non_space_char_alpha_or_digit = true;
        }
        else {
            res.push(ch);

            is_prev_non_space_char_alpha_or_digit = false;
        }

        is_prev_space = ch.is_ascii_whitespace();
    });
    res
}


#[allow(dead_code)]
pub trait StringOp {
    fn remove_space_chars(&self) -> String;
    fn is_eq_to_one_of_str<const N: usize>(&self, strings: [&str;N]) -> bool;
    fn ends_with_one_of<const N: usize>(&self, strings: [&str;N]) -> bool;
}
impl StringOp for String {
    fn remove_space_chars(&self) -> String {
        remove_space_chars_impl(self.as_str())
    }
    fn is_eq_to_one_of_str<const N: usize>(&self, strings: [&str;N]) -> bool {
        is_eq_to_one_of_str_impl(self, strings)
    }
    fn ends_with_one_of<const N: usize>(&self, strings: [&str; N]) -> bool {
        ends_with_one_of_impl(self, strings)
    }
}
impl StringOp for str {
    fn remove_space_chars(&self) -> String {
        <String as StringOp>::remove_space_chars(&self.to_string())
    }
    fn is_eq_to_one_of_str<const N: usize>(&self, strings: [&str;N]) -> bool {
        is_eq_to_one_of_str_impl(self, strings)
    }
    fn ends_with_one_of<const N: usize>(&self, strings: [&str; N]) -> bool {
        ends_with_one_of_impl(self, strings)
    }
}

pub trait OptionStringOp {
    fn is_eq_to_str(&self, str: &str) -> bool;
    fn is_eq_to_one_of_str<const N: usize>(&self, str: [&str;N]) -> bool;
}
impl OptionStringOp for Option<String> {
    fn is_eq_to_str(&self, str: &str) -> bool {
        match self {
            None => { false }
            Some(ref self_string) => { str == self_string.as_str() }
        }
    }
    fn is_eq_to_one_of_str<const N: usize>(&self, strings: [&str;N]) -> bool {
        match self {
            None => { false }
            Some(ref self_string) => {
                <Option::<&str> as OptionStringOp>::is_eq_to_one_of_str(
                    &Some(self_string.as_str()), strings)
            }
        }
    }
}
impl OptionStringOp for Option<&str> {
    fn is_eq_to_str(&self, str: &str) -> bool {
        match self {
            None => { false }
            Some(ref self_string) => { str == *self_string }
        }
    }
    fn is_eq_to_one_of_str<const N: usize>(&self, strings: [&str;N]) -> bool {
        match self {
            None => { false }
            Some(self_string) => { is_eq_to_one_of_str_impl(self_string, strings) }
        }
    }
}


fn is_eq_to_one_of_str_impl<const N: usize>(string: &str, strings: [&str;N]) -> bool {
    for str in strings {
        if string == str {
            return true;
        }
    }
    false
}

fn ends_with_one_of_impl<const N: usize>(string: &str, strings: [&str;N]) -> bool {
    for str in strings {
        if string.ends_with(str) {
            return true;
        }
    }
    false
}

pub(crate) fn keys_with_duplicates<V>(grouped_err_enum_variants_by_arg_type: &HashMap<String, Vec<V>>)
    -> HashSet<String> {
    let keys_with_duplicates: HashSet<String> = grouped_err_enum_variants_by_arg_type.iter()
        .filter(|(_, enums_vec)| enums_vec.len() > 1)
        .map(|src_type_and_vars| src_type_and_vars.0.to_string())
        .collect();
    keys_with_duplicates
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

#[derive(Debug, Display)]
#[derive(Copy, Clone)]
pub enum InternalTypePathMode {
    InternalCratePath,
    ExternalCratePath,
}

#[derive(Debug)]
pub enum InternalTypePathModeFromStrError { IncorrectInternalTypePathModeFormat }

impl FromStr for InternalTypePathMode {
    type Err = InternalTypePathModeFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "InternalCratePath" | "internal_crate_path" => Ok(InternalTypePathMode::InternalCratePath),
            "ExternalCratePath" | "external_crate_path" => Ok(InternalTypePathMode::ExternalCratePath),
            _ => Err(InternalTypePathModeFromStrError::IncorrectInternalTypePathModeFormat),
        }
    }
}


pub fn determine_internal_type_path_mode_by_macro_src_pos(_ast: &syn::DeriveInput, crate_name: &str) -> Option<InternalTypePathMode> {

    // test_compile_log();

    // simple hacky solution
    let building_crate_opt = std::env::var("CARGO_CRATE_NAME");
    use InternalTypePathMode::*;
    building_crate_opt.ok().map(|building_crate| if building_crate == crate_name { InternalCratePath } else { ExternalCratePath })

    /*
    use proc_macro_crate::FoundCrate;
    use proc_macro_crate::Error as PMCError;
    // use syn::spanned::Spanned;
    // let span = _ast.span();

    // let span = span.span();
    // let source_text = span.source_text();

    // let src_pos: Option<std::fs::> = None;

    // unstable now
    // let span = _ast.span();
    // let source_file: Option<PathBuf> = Some(span.source_file().into());
    // T O D O: impl:
    //  * if it is located in 'tests' source dir, we need to use ExternalCratePath
    //  * if it is located in 'src' source dir and current/nearest Cargo.toml [package].name = "project01", we need to use CratePath
    //  * otherwise use ExternalCratePath

    // let source_file: Option<PathBuf> = None;
    // let source_file: Option<PathBuf> = Some(PathBuf::from_str("/home/vmelnykov/projects/rust/rust-study-project-01/tests/another_static_error_macro_test.rs").unwrap());
    // let source_file: Option<PathBuf> = Some(PathBuf::from_str("/home/vmelnykov/projects/rust/rust-study-project-01/src/entity/currency.rs").unwrap());
    let is_test_source_file: bool = source_file.map(|path| is_test_source(path)).unwrap_or(false);

    let carte_name_opt: Result<FoundCrate, PMCError> = proc_macro_crate::crate_name(crate_name);

    match carte_name_opt {
        Ok(ok_res) => {
            use InternalTypePathMode::*;
            match ok_res {
                FoundCrate::Itself => { Some( if is_test_source_file { ExternalCratePath } else { CratePath } ) }
                // ?? The searched crate was found with this name.
                FoundCrate::Name(_) => { None }
            }
        }
        Err(err) => {
            use proc_macro_crate::Error::*;
            match err {
                NotFound(_) => Some(InternalTypePathMode::ExternalCratePath),
                // CargoManifestDirNotSet(..) | CargoEnvVariableNotSet(..) | FailedGettingWorkspaceManifestPath(..)
                //     | CouldNotRead(..) | InvalidToml(..) | CrateNotFound(..)
                //     => { compile_log_warn!("Cannot determine current crate: {:?}", err); None }
                _ =>  { compile_log_warn!("Cannot determine current crate: {:?}", err); None }
            }
        }
    }
    */
}

#[allow(dead_code)]
fn is_test_source(path: std::path::PathBuf) -> bool {

    if !path.exists() { return false; }

    let mut p = path.as_path();

    while let Some(ref parent) = p.parent() {

        if let Some(ref parent_file_name) = parent.file_name() {
            let is_test_dir = parent_file_name.as_encoded_bytes() == b"tests";
            let is_root_project_or_subproject_dir: bool = if is_test_dir {
                parent.with_file_name("Cargo.toml").exists()
            } else { false };

            if is_test_dir && is_root_project_or_subproject_dir {
                println!("### path [{}] belongs to 'tests' source directory", path.as_path().display());
                return true;
            }
        }

        p = parent;
    }

    println!("### path [{}] does not belongs to 'tests' source directory", path.as_path().display());
    false
}


/*
pub fn caller_crate_root() -> PathBuf {
    let crate_name =
        std::env::var("CARGO_PKG_NAME").expect("failed to read ENV var `CARGO_PKG_NAME`!");
    let current_dir = std::env::current_dir().expect("failed to unwrap env::current_dir()!");
    let search_entry = format!("name=\"{crate_name}\"");
    for entry in walkdir::WalkDir::new(&current_dir)
        .into_iter()
        .filter_entry(|e| !e.file_name().eq_ignore_ascii_case("target"))
    {
        let Ok(entry) = entry else { continue };
        if !entry.file_type().is_file() {
            continue;
        }
        let Some(file_name) = entry.path().file_name() else { continue };
        if !file_name.eq_ignore_ascii_case("Cargo.toml") {
            continue;
        }
        let Ok(cargo_toml) = std::fs::read_to_string(&entry.path()) else {
            continue
        };
        if cargo_toml
            .chars()
            .filter(|&c| !c.is_whitespace())
            .collect::<String>()
            .contains(search_entry.as_str())
        {
            return entry.path().parent().unwrap().to_path_buf();
        }
    }
    current_dir
}
*/



#[allow(dead_code)]
pub trait AddPMTokenStream {
    fn add_ts(& mut self, other_ts: proc_macro::TokenStream);
}
#[allow(dead_code)]
pub trait AddPM2TokenStream {
    fn add_pm2_ts(& mut self, other_ts: proc_macro2::TokenStream);
}
#[allow(dead_code)]
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
        remove_spaces_from_type_string(&str_type.to_owned())
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

    //noinspection RsUnresolvedPath
    #[allow(dead_code)]
    fn test_compile_log() {
        eprintln!("\n-----------------------------------------------------------------");
        println! ("### test_compile_log, println");
        eprintln!("### test_compile_log, eprintln");

        eprintln!("\n-----------------------------------------------------------------");
        compile_log_warn!("### 00 determine_internal_type_path_mode_by_macro_src_pos");
        compile_log_warn!("### 01 determine_internal_type_path_mode_by_macro_src_pos: {}", 1234);
        compile_log_warn!("### 02 determine_internal_type_path_mode_by_macro_src_pos: {} {:?}", 1234, "arg2");

        eprintln!("\n-----------------------------------------------------------------");
        compile_log_trace!("### test compile log, trace; args: {} {:?}", 1234, "arg2");
        compile_log_debug!("### test compile log, debug; args: {} {:?}", 1234, "arg2");
        compile_log_info! ("### test compile log, info ; args: {} {:?}", 1234, "arg2");
        compile_log_warn! ("### test compile log, warn ; args: {} {:?}", 1234, "arg2");
        compile_log_error!("### test compile log, error; args: {} {:?}", 1234, "arg2");
        eprintln!("\n\n");
    }
}
