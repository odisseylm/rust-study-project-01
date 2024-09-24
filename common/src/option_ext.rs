


#[extension_trait::extension_trait]
pub impl<T,E> OptionResOptExt<T,E> for Option<Result<Option<T>,E>> {
    fn flatten_opt_res_opt(self) -> Result<Option<T>,E> {
        self.unwrap_or_else(|| Ok(None))
    }
}
