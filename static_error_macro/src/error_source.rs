use crate::macro_util::type_path_to_string;


// #[derive(Debug)] // T O D O: fix/uncomment
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


impl core::fmt::Debug for ErrorSourceEnumVariant<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use syn::Type;

        let detailed_impl: Option<core::fmt::Result> = self.first_arg_type.and_then(|t| {
            if let Type::Path(ref type_path) = t {
                    Some(write!(f, "ErrorSourceEnumVariant {{ name: {}, type: {} }}", self.name, type_path_to_string(&type_path)))
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

            write!(f, "ErrorSourceEnumVariant {{ name: {}, type: {} }}", self.name, as_str)
        })
    }
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
