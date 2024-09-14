

#[derive(Debug)]
pub enum UpdateFile {
    Always,
    /// To avoid regeneration of dependant client stubs.
    IfModelChanged,
}
