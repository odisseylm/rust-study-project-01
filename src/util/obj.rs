


#[macro_export] macro_rules! generate_delegate_new_type_from_str {
    ($Type:ty, $InnerType:ty, $Err:ty) => {

        #[inherent::inherent]
        impl core::str::FromStr for $Type {
            type Err = $Err;
            pub fn from_str(str: &str) -> Result<Self, <Self as core::str::FromStr>::Err> {
                let inner_val = < $InnerType > ::from_str(str) ?;
                Ok(Self(inner_val))
            }
        }

    };
}
