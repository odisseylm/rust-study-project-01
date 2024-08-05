use crate::bt::MY_BACKTRACE_AND_ERR_CRATE;
use crate::macro_util::{
    attr_list_as_string, determine_internal_type_path_mode_by_macro_src_pos, find_attr,
    InternalTypePathMode,
};


#[derive(Debug, Clone)]
pub struct ErrorSourceEnumVariant<'a> {
    pub variant: & 'a syn::Variant,
    pub name: & 'a syn::Ident,
    pub first_arg_type: Option<& 'a syn::Type>,
}
#[derive(Debug)]
pub struct ErrorSourceEnum<'a> {
    pub name: & 'a syn::Ident,
    pub variants: Vec<ErrorSourceEnumVariant<'a>>,
}


pub fn get_error_source_enum_variants<'a>(ast: & 'a syn::DeriveInput) -> ErrorSourceEnum<'a> {
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


pub fn get_internal_type_path_mode(ast: &syn::DeriveInput) -> InternalTypePathMode {
    use core::str::FromStr;

    let internal_type_path_mode_attr = find_attr(&ast.attrs, "struct_error_internal_type_path_mode")
        .or_else(|| find_attr(&ast.attrs, "struct_error_source_internal_type_path_mode"))
        .or_else(|| find_attr(&ast.attrs, "error_source_internal_type_path_mode"))
        .or_else(|| find_attr(&ast.attrs, "internal_type_path_mode"));

    internal_type_path_mode_attr.and_then(|type_mode_attr| {
            attr_list_as_string(type_mode_attr)
                .map(|s| InternalTypePathMode::from_str(s.as_str())
                    .expect( &format!("struct_error_internal_type_path_mode has incorrect value.\
                     Possible value: {}/{}.", InternalTypePathMode::InternalCratePath, InternalTypePathMode::ExternalCratePath))
                )
        })
        .or_else(|| determine_internal_type_path_mode_by_macro_src_pos(ast, MY_BACKTRACE_AND_ERR_CRATE))
        .unwrap_or(InternalTypePathMode::InternalCratePath)
}


/*
use crate::macro_util::{ type_path_to_string };

Seems it is needed in case of integration tests in THIS subproject.

impl core::fmt::Debug for ErrorSourceEnumVariant<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use syn::Type;

        let detailed_impl: Option<core::fmt::Result> = self.first_arg_type.and_then(|t| {
            if let Type::Path(ref type_path) = t {
                    Some(write!(f, "ErrorSourceEnumVariant {{ {}, type: {} }}", self.name, type_path_to_string(&type_path)))
            } else { None }
        });

        detailed_impl.unwrap_or_else(|| {
            let as_str = match self.first_arg_type {
                None                         => { "No"          }
                Some(ref t) => {
                    match t {
                        Type::Array(_)       => { "array"       }
                        Type::BareFn(_)      => { "BareFn"      }
                        Type::Group(_)       => { "Group"       }
                        Type::ImplTrait(_)   => { "ImplTrait"   }
                        Type::Infer(_)       => { "Infer"       }
                        Type::Macro(_)       => { "Macro"       }
                        Type::Never(_)       => { "Never"       }
                        Type::Paren(_)       => { "Paren"       }
                        Type::Path(_)        => { "Path"        }
                        Type::Ptr(_)         => { "Ptr"         }
                        Type::Reference(_)   => { "Reference"   }
                        Type::Slice(_)       => { "Slice"       }
                        Type::TraitObject(_) => { "TraitObject" }
                        Type::Tuple(_)       => { "Tuple"       }
                        Type::Verbatim(_)    => { "Verbatim"    }
                        _                    => { "_"           }
                    }
                }
            };

            write!(f, "ErrorSourceEnumVariant {{ {}, type: {} }}", self.name, as_str)
        })
    }
}
*/
