

/*
use backtrace::{BacktraceFmt, BacktraceFrameFmt, PrintFmt};
use crate::util::result::PrintableResult;


Seems it is outdated third-party 'backtrace' - it generates a bit unreadable symbol names.

// Java stacktrace example:
//
// Exception in thread "main" java.lang.ArithmeticException
// 	at com.example.task01.Test.division(Test.java:10)
// 	at com.example.task01.Test.main(Test.java:6)
//
pub fn print_current_stack_trace() {
    backtrace::trace(|frame|{
        // let ip = frame.ip();
        // let symbol_address = frame.symbol_address();

        // Resolve this instruction pointer to a symbol name
        backtrace::resolve_frame(frame, |symbol| {

            let symbol_name = symbol.name()
                .map(|sn|{ sn.as_str().unwrap_or_default() }).unwrap_or_default();

            let file = symbol.filename()
                .map(|f|{ f.as_os_str() }.to_str().unwrap_or_default()).unwrap_or_default();

            let line_no = symbol.lineno().unwrap_or_default();
            let col_no = symbol.colno().unwrap_or_default();

            let is_backtrace_frame = file.contains("/src/backtrace/");
            let is_print_current_stack_trace = symbol_name.contains("print_current_stack_trace");

            if !is_backtrace_frame && !is_print_current_stack_trace {
                let prepared_path = keep_only_last_file_path_part3(file);

                // backtrace::BacktraceFmt::new(, PrintFmt::Full, )

                // BacktraceFmt::frame.fmt()
                // // println!("    at {:?}", frame);
                // // println!(BacktraceFmt::frame, frame);
                // // writeln!(BacktraceFmt::frame, "{:?}", frame);
                // format!(BacktraceFmt::frame, "{}", frame);

                //println!("    at {}", frame);

                if col_no != 0 { println!("    at {:} ({:}:{:}:{:})", symbol_name, prepared_path, line_no, col_no) }
                else { println!("    at {:} ({:}:{:})", symbol_name, prepared_path, line_no) }
            }
        });
        true
    });
}


fn keep_only_last_file_path_part3<'a>(path: &str) -> &str {
    //
    //     at _ZN9backtrace9backtrace9libunwind5trace17h69d5b72dddc2c41aE (/home/vmelnykov/.cargo/registry/src/index.crates.io-6f17d22bba15001f/backtrace-0.3.71/src/backtrace/libunwind.rs:105)
    //     at _ZN9backtrace9backtrace20trace_unsynchronized17h683c0ac602526ecdE (/home/vmelnykov/.cargo/registry/src/index.crates.io-6f17d22bba15001f/backtrace-0.3.71/src/backtrace/mod.rs:66)
    //     at _ZN9backtrace9backtrace5trace17h864d69f06aae9624E (/home/vmelnykov/.cargo/registry/src/index.crates.io-6f17d22bba15001f/backtrace-0.3.71/src/backtrace/mod.rs:53)
    //     at _ZN10error_test11test_33333317h23b761583c878bc4E (/home/vmelnykov/projects/rust/rust-study-project-01/tests/error_test.rs:58)
    //     at _ZN10error_test11test_33333328_$u7b$$u7b$closure$u7d$$u7d$17h389bb9bf7d41610bE (/home/vmelnykov/projects/rust/rust-study-project-01/tests/error_test.rs:42)
    //     at _ZN4core3ops8function6FnOnce9call_once17h49e45e6d306e6e41E (/rustc/9b00956e56009bab2aa15d7bff10916599e3d6d6/library/core/src/ops/function.rs:250)
    //
    // let last = path.find("/rust-study-project-01/")
    //     .or(path.find("/library/"));
    let last = if path.starts_with("/rustc/") { path.find("/library/") } else { None };
    return last.map(|offset|{path.split_at(offset + 1 ).1}).unwrap_or(path);
}
*/


use std::cmp::PartialEq;

pub fn print_current_stack_trace() {
    let stacktrace = std::backtrace::Backtrace::capture();

    println!("{}", stacktrace);

    // this has bad formed one-line output
    // println!("{:?}", stacktrace);
}

// It is unsafe in multithreaded
// type BSRc<T> = std::rc::Rc<T>;

// ??? It has bad performance if we really do not use backtrace or backtrace is ot used in case of 'recovering' (not failing the whole task/application).
// In my tests Arc enough fast (in the same thread) and we surely can use it for version when we capture backtrace.
//
// T O D O: how to avoid it? Probably move? Is moving thread-safe?
//
type BSRc<T> = std::sync::Arc<T>;

#[derive(Debug, PartialEq, Copy, Clone)]
enum NotCapturedInner {
    Unknown,
    Disabled,
    Unsupported,
}

fn std_backtrace_status_to_inner_not_captured(b: &std::backtrace::Backtrace) -> Option<NotCapturedInner> {
    use std::backtrace::BacktraceStatus;
    match b.status() {
        BacktraceStatus::Captured    => { None }
        BacktraceStatus::Unsupported => { Some(NotCapturedInner::Unsupported) }
        BacktraceStatus::Disabled    => { Some(NotCapturedInner::Disabled)    }
        _ => { Some(NotCapturedInner::Unknown) }
    }
}


pub enum NewBacktracePolicy {
    Default,
    NoBacktrace,
    Capture,
    ForceCapture,
}

// should be used together with other/source/from Error
pub enum InheritBacktracePolicy {
    Default,
    Inherit,
    InheritOrCapture,
    InheritOrForceCapture,
}
// should be used together with other/source/from Error
// pub enum BorrowBacktracePolicy {
//     Default,
//     Borrow,
//     BorrowOrCapture,
//     BorrowOrForceCapture,
// }


pub struct BacktraceInfo {
    not_captured: Option<NotCapturedInner>,
    inner: Option<BSRc<Inner>>,
}

struct Inner {
    std_backtrace: Option<std::backtrace::Backtrace>,
    str_backtrace: Option<String>,
}

impl core::fmt::Display for Inner {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if let Some(ref std_bt) = self.std_backtrace {
            write!(f, "{}", std_bt) }
        else if let Some(ref str_bt) = self.str_backtrace {
            write!(f, "{}", str_bt) }
        else {
            write!(f, "No backtrace")
        }
    }
}


pub enum BacktraceKind<'a> {
    System(& 'a std::backtrace::Backtrace),
    AsString(& 'a String),
    NoAnyBacktrace,
}

impl core::fmt::Display for BacktraceKind<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            BacktraceKind::System(std_bt)   => { write!(f, "{}", std_bt)   }
            BacktraceKind::AsString(str_bt) => { write!(f, "{}", str_bt)   }
            BacktraceKind::NoAnyBacktrace   => { write!(f, "No backtrace") }
        }
    }
}
impl core::fmt::Debug for BacktraceKind<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self)
    }
}


// pub trait BacktraceRefProvider {
//     fn provide_backtrace(&self) -> &BacktraceInfo;
// }
pub trait BacktraceCopyProvider {
    // Using 'provide' name causes warning 'unstable_name_collision'
    fn provide_backtrace(&self) -> BacktraceInfo;
    fn contains_self_or_child_captured_backtrace(&self) -> bool {
        self.provide_backtrace().is_captured()
    }
}
pub trait BacktraceBorrowedProvider { // or better Moved???
    fn provide_backtrace(&self) -> BacktraceInfo;
}


// impl std::error::Error for Error {
//     fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {
//         request.provide_ref::<MyBacktrace>(&self.backtrace);
//     }
// }


// static DISABLED_BACKTRACE: std::backtrace::Backtrace = std::backtrace::Backtrace::disabled();
// static BACKTRACE_STATUS_DISABLED: std::backtrace::BacktraceStatus = std::backtrace::BacktraceStatus::Disabled;
// static BACKTRACE_STATUS_CAPTURED: std::backtrace::BacktraceStatus = std::backtrace::BacktraceStatus::Captured;
// static BACKTRACE_STATUS_UNSUPPORTED: std::backtrace::BacktraceStatus = std::backtrace::BacktraceStatus::Unsupported;


impl BacktraceInfo {
    #[inline]
    pub fn new() -> Self { Self::new_by_policy(NewBacktracePolicy::Default) }

    pub fn from_str(str: &str) -> Self { Self::from_string(str.to_string()) }

    pub fn from_string(string: String) -> Self {
        let str_bt = string;
        let not_captured_status: Option<NotCapturedInner> =
            if !is_str_backtrace_captured(&str_bt) { Some(NotCapturedInner::Unknown) }
            else { None };

        BacktraceInfo {
            not_captured: not_captured_status,
            inner: Some(BSRc::new(Inner { std_backtrace: None, str_backtrace: Some(str_bt) })),
        }
    }

    pub fn new_by_policy(backtrace_policy: NewBacktracePolicy) -> Self {
        use NewBacktracePolicy::*;
        match backtrace_policy {
            Default       => { Self::capture_by_default() }
            Capture       => { Self::capture() }
            NoBacktrace   => { Self::empty() }
            ForceCapture  => { Self::force_capture() }
        }
    }

    #[inline]
    pub fn inherit_from<BP: BacktraceCopyProvider>(source: &BP) -> Self {
        Self::inherit_with_policy(source, InheritBacktracePolicy::Default)
    }

    pub fn inherit_with_policy<BP: BacktraceCopyProvider>(source: &BP, backtrace_policy: InheritBacktracePolicy) -> Self {
        Self::reuse(source.provide_backtrace(), backtrace_policy)
    }

    #[inline]
    pub fn borrow_from<BP: BacktraceBorrowedProvider>(source: &BP) -> Self {
        Self::borrow_with_policy(source, InheritBacktracePolicy::Default)
    }

    pub fn borrow_with_policy<BP: BacktraceBorrowedProvider>(source: &BP, backtrace_policy: InheritBacktracePolicy) -> Self {
        Self::reuse(source.provide_backtrace(), backtrace_policy)
    }

    fn reuse(source_bt: BacktraceInfo, backtrace_policy: InheritBacktracePolicy) -> Self {
        use InheritBacktracePolicy::*;
        match backtrace_policy {
            Inherit                     => { source_bt }
            Default | InheritOrCapture  => { if source_bt.is_captured() { source_bt } else { Self::new_by_policy(NewBacktracePolicy::Default)      } }
            InheritOrForceCapture       => { if source_bt.is_captured() { source_bt } else { Self::new_by_policy(NewBacktracePolicy::ForceCapture) } }
        }
    }

    pub fn is_captured(&self) -> bool { self.not_captured.is_none() && self.inner.is_some() }

    // T O D O: probably it is behavior should be configured
    fn capture_by_default() -> Self { Self::force_capture() }

    pub fn capture() -> Self {
        let std_bt = std::backtrace::Backtrace::capture();
        let not_captured_status: Option<NotCapturedInner> = std_backtrace_status_to_inner_not_captured(&std_bt);

        BacktraceInfo {
            not_captured: not_captured_status,
            inner: Some(BSRc::new(Inner { std_backtrace: Some(std_bt), str_backtrace: None })),
        }
    }

    pub fn force_capture() -> Self {
        let std_bt = std::backtrace::Backtrace::force_capture();
        let not_captured_status: Option<NotCapturedInner> = std_backtrace_status_to_inner_not_captured(&std_bt);

        BacktraceInfo {
            not_captured: not_captured_status,
            inner: Some(BSRc::new(Inner { std_backtrace: Some(std_bt), str_backtrace: None })),
        }
    }

    #[inline]
    pub fn empty() -> Self { Self::disabled() }

    pub fn disabled() -> Self {
        BacktraceInfo {
            not_captured: Some(NotCapturedInner::Disabled),
            inner: None,
        }
    }

    // T O D O: Do not use it manually. Use something like: self.inherit(), self.inherit_or_capture()/self.new_or()
    // #[deprecated(note = "mainly for internal or automatic usage when container is cloned.")]
    pub fn clone(&self) -> Self {
        if let Some(not_captured) = self.not_captured {
            BacktraceInfo { not_captured: Some(not_captured), inner: None }
        }
        else if let Some(ref inner) = self.inner {
            BacktraceInfo{ not_captured: None, inner: Some(BSRc::clone(inner)) }
        }
        else {
            BacktraceInfo { not_captured: Some(NotCapturedInner::Unknown), inner: None }
        }
    }

    // We cannot return enum copy there since this enum is 'non_exhaustive'
    // and does not support 'copy/clone'.
    pub fn backtrace_status(&self) -> std::backtrace::BacktraceStatus {
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
    }

    pub fn backtrace(&self) -> BacktraceKind {
        if !self.is_captured() { BacktraceKind::NoAnyBacktrace } // small optimization, probably unneeded
        else if let Some(ref ptr_backtrace) = self.inner {
            let std_bt = ptr_backtrace.std_backtrace.as_ref().map(|std_bt| BacktraceKind::System(std_bt));
            let str_bt = ptr_backtrace.str_backtrace.as_ref().map(|str_bt| BacktraceKind::AsString(str_bt) );
            std_bt.or(str_bt).unwrap_or(BacktraceKind::NoAnyBacktrace)
        }
        else { BacktraceKind::NoAnyBacktrace }
    }
    pub fn std_backtrace(&self) -> Option<&std::backtrace::Backtrace> {
        if let BacktraceKind::System(ref std_bt) = self.backtrace() {
            Some(std_bt)
        } else { None }

        /*
        if let Some(not_captured) = self.not_captured {
            return match not_captured {
                NotCapturedInner::Disabled =>    { &DISABLED_BACKTRACE }
                // bad approach..., but cheap
                NotCapturedInner::Unknown =>     { &DISABLED_BACKTRACE }
                NotCapturedInner::Unsupported => { &DISABLED_BACKTRACE }
            }
        }

        else if let Some(ref ptr_backtrace) = self.inner {
            return &ptr_backtrace
        }

        // bad approach..., but cheap
        return &DISABLED_BACKTRACE;
        */
    }
}


fn is_str_backtrace_captured(str_backtrace: &str) -> bool {
    str_backtrace.contains('\n')
}

impl std::fmt::Debug for BacktraceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::backtrace::*;

        match self.backtrace_status() {
            BacktraceStatus::Unsupported => { write!(f, "Backtrace unsupported") }
            BacktraceStatus::Disabled    => { write!(f, "Backtrace disabled")    }
            BacktraceStatus::Captured    => {
                match self.inner {
                    Some(ref inner) => { write!(f, "\n{}", inner) }
                    None => { write!(f, "Unknown backtrace.") }
                }
            }
            _ => { write!(f, "Unknown backtrace.") }
        }
    }
}


impl std::fmt::Display for BacktraceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// -------------------------------------------------------------------------------------------------
//                           BacktraceProvider for standard types
// -------------------------------------------------------------------------------------------------

impl BacktraceCopyProvider for anyhow::Error {
    fn provide_backtrace(&self) -> BacktraceInfo {
        BacktraceInfo::from_string(self.backtrace().to_string())
    }
}


fn std_backtrace_of_std_err<'a>(_err: & 'a dyn std::error::Error) -> Option<& 'a std::backtrace::Backtrace> {
    // TODO: add support of it after appearing std::error::Error.provide() in stable build.
    None
}

impl BacktraceCopyProvider for Box<dyn std::error::Error> {
    fn provide_backtrace(&self) -> BacktraceInfo {
        Some(self.as_ref()).provide_backtrace()
    }

    fn contains_self_or_child_captured_backtrace(&self) -> bool {
        Some(self.as_ref()).contains_self_or_child_captured_backtrace()
    }
}

impl<'a> BacktraceCopyProvider for Option<& 'a dyn std::error::Error> {
    fn provide_backtrace(&self) -> BacktraceInfo {
        let std_err_opt = self.and_then(|err| std_backtrace_of_std_err(err));
        // TODO: do not use string by performance reason
        // TODO: add warn logging
        std_err_opt.map(|bt| BacktraceInfo::from_string(bt.to_string())).unwrap_or(BacktraceInfo::empty())
    }

    fn contains_self_or_child_captured_backtrace(&self) -> bool {
        let std_err_opt = self.and_then(|err| std_backtrace_of_std_err(err));
        std_err_opt.map(|bt| bt.status() == std::backtrace::BacktraceStatus::Captured).unwrap_or(false)
    }
}

impl BacktraceCopyProvider for String {
    fn provide_backtrace(&self) -> BacktraceInfo { BacktraceInfo::empty() }
    fn contains_self_or_child_captured_backtrace(&self) -> bool { false }
}
impl BacktraceCopyProvider for &String {
    fn provide_backtrace(&self) -> BacktraceInfo { BacktraceInfo::empty() }
    fn contains_self_or_child_captured_backtrace(&self) -> bool { false }
}
impl BacktraceCopyProvider for &str {
    fn provide_backtrace(&self) -> BacktraceInfo { BacktraceInfo::empty() }
    fn contains_self_or_child_captured_backtrace(&self) -> bool { false }
}

impl BacktraceCopyProvider for i32 {
    fn provide_backtrace(&self) -> BacktraceInfo { BacktraceInfo::empty() }
    fn contains_self_or_child_captured_backtrace(&self) -> bool { false }
}

impl BacktraceCopyProvider for &i32 {
    fn provide_backtrace(&self) -> BacktraceInfo { BacktraceInfo::empty() }
    fn contains_self_or_child_captured_backtrace(&self) -> bool { false }
}



// -------------------------------------------------------------------------------------------------
//                               Enable/disable backtrace
// -------------------------------------------------------------------------------------------------

// T O D O: seems it do NOT work at all or are not stable
pub fn enable_backtrace() {
    is_anyhow_backtrace_enabled(); // to init as early as possible, TODO: make it auto-initialized

    let to_enable_value = "full"; // or "1" or "full" ??
    let rust_backtrace_cur_value: String = std::env::var("RUST_BACKTRACE").unwrap_or("".to_string());

    if rust_backtrace_cur_value != to_enable_value {
        std::env::set_var("RUST_BACKTRACE", to_enable_value);
    }
}

// T O D O: seems it do NOT work at all or are not stable
pub fn disable_backtrace() {
    is_anyhow_backtrace_enabled(); // to init as early as possible,

    // std::env::set_var("RUST_BACKTRACE", "0");
    std::env::remove_var("RUST_BACKTRACE");
}

// static INITIAL_RUST_BACKTRACE_ENABLED: Lazy<bool> = Lazy::new(|| std::env::var("RUST_BACKTRACE").map(|v| v == "1" || v == "false").unwrap_or(false));
// static INITIAL_RUST_BACKTRACE_ENABLED: Lazy<bool> = Lazy::new(|| std::env::var("RUST_BACKTRACE").map(|v| v == "1" || v == "false").unwrap_or(false));
// static INITIAL_RUST_BACKTRACE_ENABLED: bool = std::env::var("RUST_BACKTRACE").map(|v| v == "1" || v == "false").unwrap_or(false);


// static INITIAL_RUST_BACKTRACE_ENABLED: once_cell::sync::Lazy<bool> = once_cell::sync::Lazy::new(||
//     {println!("### FUCK Lazy");
//     std::env::var("RUST_BACKTRACE").map(|v| v == "1" || v == "false").unwrap_or(false)});

// static INITIAL_RUST_BACKTRACE_ENABLED: once_cell::sync::Lazy<bool> = once_cell::sync::Lazy::new(|| is_anyhow_backtrace_enabled_impl());
static INITIAL_RUST_BACKTRACE_ENABLED: once_cell::sync::Lazy<bool> = once_cell::sync::Lazy::new(|| {
    println!("### Initializing of INITIAL_RUST_BACKTRACE_ENABLED!!!"); // T O D O: remove after test stabilization
    is_anyhow_backtrace_enabled_impl()
});


// use lazy_static::lazy_static;
// lazy_static! {
//     static ref INITIAL_RUST_BACKTRACE_ENABLED: bool = std::env::var("RUST_BACKTRACE").map(|v| v == "1" || v == "false").unwrap_or(false);
// }


pub fn is_anyhow_backtrace_enabled() -> bool {
    *INITIAL_RUST_BACKTRACE_ENABLED
}


fn is_anyhow_backtrace_enabled_impl() -> bool {
    use std::fmt::Write;

    let res: Result<i32, & 'static str> = Err("test error");
    let anyhow_res: Result<i32, anyhow::Error> = res.map_err(anyhow::Error::msg);

    let mut str_buf = String::new();
    let write_res = write!(str_buf, "{:?}", anyhow_res);

    let is_backtrace_present = if write_res.is_ok() {
        str_buf.contains("Stack backtrace") || str_buf.contains("is_anyhow_backtrace_enabled") }
        else { false };
    is_backtrace_present
}
