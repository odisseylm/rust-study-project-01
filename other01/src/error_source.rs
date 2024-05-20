

#[derive(Debug)]
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
