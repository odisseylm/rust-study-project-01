


#[macro_export] macro_rules! generate_from_str_new_type_delegate {
    ($Type:ty, $InnerType:ty, $Err:ty) => {

        #[inherent::inherent]
        impl core::str::FromStr for $Type {
            type Err = $Err;
            #[inline]
            pub fn from_str(str: &str) -> Result<Self, <Self as core::str::FromStr>::Err> {
                use core::str::FromStr;
                let inner_val = < $InnerType > ::from_str(str) ?;
                Ok(Self(inner_val))
            }
        }

    };

    ($Type:ty, $InnerType:ty, $parse_func:ident, $Err:ty) => {

        #[inherent::inherent]
        impl core::str::FromStr for $Type {
            type Err = $Err;
            #[inline]
            pub fn from_str(str: &str) -> Result<Self, <Self as core::str::FromStr>::Err> {
                let inner_val = < $InnerType > :: $parse_func (str) ?;
                Ok(Self(inner_val))
            }
        }

    };
}
