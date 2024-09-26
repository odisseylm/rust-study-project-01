use core::fmt::{self, Display, Formatter};
//--------------------------------------------------------------------------------------------------



// We cannot implement Display for core::Option due to rust design
// Let's use our similar type.
//
pub enum OptionRefOnlySomeDisplay<'a, T: Display> {
    None,
    Some(&'a T),
}
impl<'a, T: Display> Display for OptionRefOnlySomeDisplay<'a, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OptionRefOnlySomeDisplay::None =>
                Ok(()),
            OptionRefOnlySomeDisplay::Some(val_ref) =>
                <T as Display>::fmt(*val_ref, f),
        }
    }
}


#[extension_trait::extension_trait]
pub impl<T> DisplayOptionExt<T> for Option<T> {
    #[allow(unused_attributes)]
    #[inline]
    fn as_only_some_to_display(&self) -> OptionRefOnlySomeDisplay<T>
        where T: Display {
        match self {
            None =>
                OptionRefOnlySomeDisplay::None,
            Some(ref ref_val) =>
                OptionRefOnlySomeDisplay::Some(ref_val),
        }
    }
}


#[extension_trait::extension_trait]
pub impl<T,E> OptionResOptExt<T,E> for Option<Result<Option<T>,E>> {
    // Long name if 'flatten' causes names conflicts
    #[allow(unused_attributes)]
    #[inline]
    fn flatten_opt_res_opt(self) -> Result<Option<T>,E> {
        self.unwrap_or_else(|| Ok(None))
    }
    #[allow(unused_attributes)]
    #[inline]
    fn flatten(self) -> Result<Option<T>,E> {
        self.flatten_opt_res_opt()
    }
}
