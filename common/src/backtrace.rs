use core::fmt::{ self, Debug, Display };
// use std::sync::Arc;
//--------------------------------------------------------------------------------------------------


pub type BacktraceCell = BacktraceCell_PtrCellImpl;
// pub type BacktraceCell = BacktraceCell_ArcImpl;
// pub type BacktraceSource = BacktraceSourceInternal<BacktraceCell_ArcImpl>;
// trait BacktraceSource = BacktraceSourceInternal<BacktraceCell>;

impl From<BacktraceCell> for std::backtrace::Backtrace {
    fn from(value: BacktraceCell) -> Self {
        value.move_out().take_std_backtrace()
    }
}

/*
enum BacktraceInner {
    Std(std::backtrace::Backtrace),
    // AnyhowError(anyhow::Error),
}
*/

impl BacktraceCell {
    pub fn inherit_or_capture<Src: BacktraceSource>(src: &Src) -> Self {
        if src.contains_backtrace() {
            if src.is_taking_backtrace_supported() {
                let bt = src.take_backtrace();
                if bt.is_empty() {
                    Self::capture_backtrace()
                } else {
                    bt
                }
            } else {
                Self::capture_backtrace()
            }
        } else {
            Self::capture_backtrace()
        }
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
}


/*
pub trait BacktraceCellContract /*: Sized */ {
    fn empty() -> Self;
    fn is_empty(&self) -> bool;
    fn with_backtrace(backtrace: std::backtrace::Backtrace) -> Self;
    fn move_out(&self) -> Self;

    fn new(backtrace: Option<std::backtrace::Backtrace>) -> Self;
    fn capture_backtrace() -> Self;
    fn inherit_or_capture<Src: BacktraceSourceInternal<Self>>(src: &Src) -> Self;
}
trait BacktraceCellDefaultImpl: BacktraceCellContract + Sized {
    fn new_impl(backtrace: Option<std::backtrace::Backtrace>) -> Self {
        match backtrace {
            None => <Self as BacktraceCellContract>::empty(),
            Some(backtrace) => <Self as BacktraceCellContract>::with_backtrace(backtrace)
        }
    }
    fn capture_backtrace_impl() -> Self {
        Self::with_backtrace(std::backtrace::Backtrace::capture())
    }

    fn inherit_or_capture_impl<Src: BacktraceSourceInternal<Self>>(src: &Src) -> Self {
        let mut bt = src.take_backtrace();
        if bt.is_empty() {
            bt = <Self as BacktraceCellContract>::capture_backtrace()
        }
        bt
    }
}
*/


pub trait BacktraceSource {
    fn backtrace_ref(&self) -> Option<&BacktraceCell>;

    /// It can verify ANY backtrace including child error.
    ///
    fn contains_backtrace(&self) -> bool {
        match self.backtrace_ref() {
            None => false,
            Some(bt) => !bt.is_empty(),
        }
    }

    fn is_taking_backtrace_supported(&self) -> bool;

    /// It is unusual behavior for general types to 'take' (move out)
    /// field by immutable ref
    /// (usually mutable ref or even passing by value is used for this purpose).
    /// BUT it is really intentional behavior to use interior mutability
    /// to easy pass backtrace (without changing other error details)
    /// to top level error.
    ///
    fn take_backtrace(&self) -> BacktraceCell {
        match self.backtrace_ref() {
            None => BacktraceCell::empty(),
            Some(bt) => bt.move_out(),
        }
    }
}

impl BacktraceSource for anyhow::Error {
    fn backtrace_ref(&self) -> Option<&BacktraceCell> {
        None
    }

    fn contains_backtrace(&self) -> bool {
        if let std::backtrace::BacktraceStatus::Captured = self.backtrace().status() { true }
        else { false }
    }

    fn is_taking_backtrace_supported(&self) -> bool {
        false
    }

    fn take_backtrace(&self) -> BacktraceCell {
        BacktraceCell::empty()
    }
}

impl BacktraceSource for Box<dyn std::error::Error> {
    fn backtrace_ref(&self) -> Option<&BacktraceCell> {
        None
    }

    fn contains_backtrace(&self) -> bool {
        // To implement it we need to use nightly build
        false

        /*
        if let BacktraceStatus::Captured = self.backtrace().status() { true }
        else { false }
        */
    }

    fn is_taking_backtrace_supported(&self) -> bool {
        false
    }

    fn take_backtrace(&self) -> BacktraceCell {
        BacktraceCell::empty()
    }
}


impl BacktraceSource for BacktraceCell {
    fn backtrace_ref(&self) -> Option<&BacktraceCell> {
        Some(self)
    }

    fn contains_backtrace(&self) -> bool {
        self.backtrace_status() != std::backtrace::BacktraceStatus::Captured
    }

    fn is_taking_backtrace_supported(&self) -> bool {
        !self.is_empty()
    }

    fn take_backtrace(&self) -> BacktraceCell {
        self.move_out()
    }
}


#[allow(non_camel_case_types)]
pub struct BacktraceCell_PtrCellImpl {
    cell: ptr_cell::PtrCell<std::backtrace::Backtrace>,
}

// impl BacktraceCellDefaultImpl for BacktraceCell_PtrCellImpl { }

// #[inherent::inherent]
impl BacktraceCell_PtrCellImpl {
    #[inline]
    pub fn empty() -> Self {
        Self { cell: ptr_cell::PtrCell::new(None) }
    }
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.cell.is_empty(ptr_cell::Semantics::Relaxed)
    }
    #[inline]
    pub fn with_backtrace(backtrace: std::backtrace::Backtrace) -> Self {
        Self { cell: ptr_cell::PtrCell::new(Some(backtrace)) }
    }

    pub fn move_out(&self) -> Self {
        match self.cell.take(ptr_cell::Semantics::Relaxed) {
            None => Self::empty(),
            Some(bt) => Self::with_backtrace(bt),
        }
    }

    pub fn take_std_backtrace(&self) -> std::backtrace::Backtrace {
        self.cell.take(ptr_cell::Semantics::Relaxed)
            .unwrap_or_else(|| std::backtrace::Backtrace::disabled())
    }

    //
    // fn inherit_or_capture_impl<Src: BacktraceSource>(src: &Src) -> Self {
    //     let mut bt = src.take_backtrace();
    //     if bt.is_empty() {
    //         bt = Self::capture_backtrace()
    //     }
    //     bt
    // }

    // T O D O: for backward compatibility, REMOVE it later, I do not think that we need it.
    //
    // We cannot return enum copy there since this enum is 'non_exhaustive'
    // and does not support 'copy/clone'.
    pub fn backtrace_status(&self) -> std::backtrace::BacktraceStatus {

        let bt = self.cell.take(ptr_cell::Semantics::Relaxed);
        let status = match bt {
            None => std::backtrace::BacktraceStatus::Disabled,
            Some(ref bt) => bt.status(),
        };
        self.cell.set(bt, ptr_cell::Semantics::Relaxed);
        status

        /*
        if let Some(not_captured) = self.not_captured {
            match not_captured {
                NotCapturedInner::Disabled    => { std::backtrace::BacktraceStatus::Disabled    }
                NotCapturedInner::Unsupported => { std::backtrace::BacktraceStatus::Unsupported }
                NotCapturedInner::Unknown     => { std::backtrace::BacktraceStatus::Unsupported }
            }
        }
        else if let Some(ref ptr_backtrace) = self.inner {
            let std_bt_status = ptr_backtrace.std_backtrace.as_ref().map(|std_bt| std_bt.status());

            let str_bt_status = ptr_backtrace.str_backtrace.as_ref()
                .map(|str_bt|
                if is_str_backtrace_captured(str_bt) { std::backtrace::BacktraceStatus::Captured }
                else { std::backtrace::BacktraceStatus::Disabled } );

            std_bt_status.or(str_bt_status).unwrap_or(std::backtrace::BacktraceStatus::Disabled)
        }
        else {
            std::backtrace::BacktraceStatus::Unsupported
        }
        */
    }

}

impl Debug for BacktraceCell_PtrCellImpl {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        <Self as Display>::fmt(self, f)
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
/*
#[allow(non_camel_case_types)]
pub struct BacktraceCell_ArcImpl {
    cell: Arc<Option<std::backtrace::Backtrace>>,
}

// impl BacktraceCellDefaultImpl for BacktraceCell_ArcImpl { }

// #[inherent::inherent]
impl BacktraceCell_ArcImpl {
    pub fn empty() -> Self {
        Self { cell: Arc::new(None) }
    }
    pub fn is_empty(&self) -> bool {
        self.cell.is_none()
    }
    pub fn with_backtrace(backtrace: std::backtrace::Backtrace) -> Self {
        Self { cell: Arc::new(Some(backtrace)) }
    }
    // Actually it is impossible to move out immutable Arc content
    pub fn move_out(&self) -> Self {
        let copy = Arc::clone(&self.cell);
        // self.cell. ; ?? how to move/clear immutable Arc ??
        Self { cell: copy }
    }

    pub fn take_std_backtrace(&self) -> std::backtrace::Backtrace {
        match self.cell.as_ref() {
            None => std::backtrace::Backtrace::disabled(),
            Some(bt) => bt.
        }
    }

    // TODO: for backward compatibility, REMOVE it later, I do not think that we need it.
    //
    // We cannot return enum copy there since this enum is 'non_exhaustive'
    // and does not support 'copy/clone'.
    pub fn backtrace_status(&self) -> std::backtrace::BacktraceStatus {

        let bt = self.cell.as_ref();
        let status = match bt {
            None => std::backtrace::BacktraceStatus::Disabled,
            Some(ref bt) => bt.status(),
        };
        status

        /*
        if let Some(not_captured) = self.not_captured {
            match not_captured {
                NotCapturedInner::Disabled    => { std::backtrace::BacktraceStatus::Disabled    }
                NotCapturedInner::Unsupported => { std::backtrace::BacktraceStatus::Unsupported }
                NotCapturedInner::Unknown     => { std::backtrace::BacktraceStatus::Unsupported }
            }
        }
        else if let Some(ref ptr_backtrace) = self.inner {
            let std_bt_status = ptr_backtrace.std_backtrace.as_ref().map(|std_bt| std_bt.status());

            let str_bt_status = ptr_backtrace.str_backtrace.as_ref()
                .map(|str_bt|
                if is_str_backtrace_captured(str_bt) { std::backtrace::BacktraceStatus::Captured }
                else { std::backtrace::BacktraceStatus::Disabled } );

            std_bt_status.or(str_bt_status).unwrap_or(std::backtrace::BacktraceStatus::Disabled)
        }
        else {
            std::backtrace::BacktraceStatus::Unsupported
        }
        */
    }

}


impl Debug for BacktraceCell_ArcImpl {
    #[inline]
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
*/


//--------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {

    /*
    mod arc_impl_test {
        pub type BacktraceCell = super::super::BacktraceCell_ArcImpl;

        #[allow(dead_code)]
        async fn compile_test_1() {
            // non-mutable
            let _c1 = BacktraceCell::with_backtrace(std::backtrace::Backtrace::capture());
        }

        #[allow(dead_code)]
        async fn async_fn<T: Send + Sync>(v: T) -> T {
            v
        }

        //noinspection DuplicatedCode
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
            // for Arc it does not work
            // and I do not know how to fix it in cheap way without Mutex/RwLock
            // assert!( c1.is_empty());
            assert!(!c2.is_empty());
        }
    }
    */

    mod ptr_cell_impl_test {
        pub type BacktraceCell = super::super::BacktraceCell_PtrCellImpl;

        #[allow(dead_code)]
        async fn compile_test_1() {
            // non-mutable
            let _c1 = BacktraceCell::with_backtrace(std::backtrace::Backtrace::capture());
        }

        #[allow(dead_code)]
        async fn async_fn<T: Send + Sync>(v: T) -> T {
            v
        }

        //noinspection DuplicatedCode
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
            assert!(c1.is_empty());
            assert!(!c2.is_empty());
        }
    }
}
