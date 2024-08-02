use core::fmt::{ self, Debug, Display };
use std::sync::Arc;
//--------------------------------------------------------------------------------------------------


// pub type BacktraceCell = BacktraceCell_PtrCellImpl;
pub type BacktraceCell = BacktraceCell_ArcImpl;


#[allow(non_camel_case_types)]
pub struct BacktraceCell_PtrCellImpl {
    cell: ptr_cell::PtrCell<std::backtrace::Backtrace>,
}
impl BacktraceCell_PtrCellImpl {
    pub fn empty() -> Self {
        Self { cell: ptr_cell::PtrCell::new(None) }
    }
    pub fn with_backtrace(backtrace: std::backtrace::Backtrace) -> Self {
        Self { cell: ptr_cell::PtrCell::new(Some(backtrace)) }
    }
    pub fn new(backtrace: Option<std::backtrace::Backtrace>) -> Self {
        match backtrace {
            None => Self::empty(),
            Some(backtrace) => Self::with_backtrace(backtrace)
        }
    }
    pub fn capture_backtrace() -> Self {
        Self::with_backtrace(std::backtrace::Backtrace::capture())
    }
    pub fn is_empty(&self) -> bool {
        self.cell.is_empty(ptr_cell::Semantics::Relaxed)
    }
    pub fn move_out(&self) -> Self {
        match self.cell.take(ptr_cell::Semantics::Relaxed) {
            None => Self::empty(),
            Some(bt) => Self::with_backtrace(bt),
        }
    }
}

impl Debug for BacktraceCell_PtrCellImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bt = self.cell.take(ptr_cell::Semantics::Relaxed);
        let res = match bt {
            None =>
                <str as Debug>::fmt("", f),
            Some(ref bt) =>
                <std::backtrace::Backtrace as Debug>::fmt(bt, f),
        };

        // ptr_cell does not have safe 'get ref' method
        self.cell.set(bt, ptr_cell::Semantics::Relaxed);
        res
    }
}
impl Display for BacktraceCell_PtrCellImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bt = self.cell.take(ptr_cell::Semantics::Relaxed);
        let res = match bt {
            None =>
                <str as Display>::fmt("", f),
            Some(ref bt) =>
                <std::backtrace::Backtrace as Display>::fmt(bt, f),
        };

        // ptr_cell does not have safe 'get ref' method
        self.cell.set(bt, ptr_cell::Semantics::Relaxed);
        res
    }
}


//--------------------------------------------------------------------------------------------------
#[allow(non_camel_case_types)]
#[derive(Clone)]
pub struct BacktraceCell_ArcImpl {
    cell: Arc<Option<std::backtrace::Backtrace>>,
}
impl BacktraceCell_ArcImpl {
    pub fn empty() -> Self {
        Self { cell: Arc::new(None) }
    }
    pub fn with_backtrace(backtrace: std::backtrace::Backtrace) -> Self {
        Self { cell: Arc::new(Some(backtrace)) }
    }
    pub fn new(backtrace: Option<std::backtrace::Backtrace>) -> Self {
        match backtrace {
            None => Self::empty(),
            Some(backtrace) => Self::with_backtrace(backtrace)
        }
    }
    pub fn capture_backtrace() -> Self {
        Self::with_backtrace(std::backtrace::Backtrace::capture())
    }
    pub fn is_empty(&self) -> bool {
        self.cell.is_none()
    }
    // Actually it is impossible to move out immutable Arc content
    pub fn move_out(&self) -> Self {
        let copy = Arc::clone(&self.cell);
        // self.cell. ; ?? how to move/clear immutable Arc ??
        Self { cell: copy }
    }
}


impl Debug for BacktraceCell_ArcImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as Display>::fmt(self, f)
    }
}
impl Display for BacktraceCell_ArcImpl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.cell.as_ref() {
            None =>
                <str as Display>::fmt("", f),
            Some(ref bt) =>
                <std::backtrace::Backtrace as Display>::fmt(bt, f),
        }
    }
}


//--------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    async fn compile_test_1() {
        // non-mutable
        let _c1 = BacktraceCell::with_backtrace(std::backtrace::Backtrace::capture());
    }

    #[allow(dead_code)]
    async fn async_fn <T: Send + Sync > (v: T) -> T {
        v
    }

    #[allow(dead_code)]
    async fn compile_test_for_multi_threads() {
        let bt = std::backtrace::Backtrace::capture();
        let _ = async_fn(bt).await;

        let bt = Some(std::backtrace::Backtrace::capture());
        let _ = async_fn(bt).await;

        // non-mutable
        let c1 = BacktraceCell::with_backtrace(std::backtrace::Backtrace::capture());
        let _ = async_fn(c1).await;
    }

    #[test]
    fn test1() {
        // non-mutable
        let c1 = BacktraceCell::with_backtrace(std::backtrace::Backtrace::capture());
        assert!(!c1.is_empty());

        //
        let c2 = c1.move_out();
        assert!( c1.is_empty());
        assert!(!c2.is_empty());
    }
}
