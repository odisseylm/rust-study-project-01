


#[macro_export] macro_rules! generate_into_inner_delegate {
    ($Type:ty, $InnerType:ty) => {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl $Type {
            #[inline]
            pub fn into_inner(self) -> $InnerType {
                self.0
            }
        }

    };
    ($Type:ty, $field:ident, $InnerType:ty) => {

        #[allow(unused_imports)]
        #[allow(unused_qualifications)]
        impl $Type {
            #[inline]
            pub fn into_inner(self) -> $InnerType {
                self. $field
            }
        }

    };
}

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
