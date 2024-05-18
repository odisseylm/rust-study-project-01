

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
    inner: Option<BSRc<std::backtrace::Backtrace>>,
}

// pub trait BacktraceRefProvider {
//     fn provide_backtrace(&self) -> &BacktraceInfo;
// }
pub trait BacktraceCopyProvider {
    // Using 'provide' name causes warning 'unstable_name_collision'
    fn provide_backtrace(&self) -> BacktraceInfo;
}
pub trait BacktraceBorrowedProvider { // or better Moved???
    fn provide_backtrace(&self) -> BacktraceInfo;
}


// impl std::error::Error for Error {
//     fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {
//         request.provide_ref::<MyBacktrace>(&self.backtrace);
//     }
// }


static DISABLED_BACKTRACE: std::backtrace::Backtrace = std::backtrace::Backtrace::disabled();
// static BACKTRACE_STATUS_DISABLED: std::backtrace::BacktraceStatus = std::backtrace::BacktraceStatus::Disabled;
// static BACKTRACE_STATUS_CAPTURED: std::backtrace::BacktraceStatus = std::backtrace::BacktraceStatus::Captured;
// static BACKTRACE_STATUS_UNSUPPORTED: std::backtrace::BacktraceStatus = std::backtrace::BacktraceStatus::Unsupported;


impl BacktraceInfo {
    #[inline]
    pub fn new() -> Self { Self::new_by_policy(NewBacktracePolicy::Default) }

    pub fn new_by_policy(backtrace_policy: NewBacktracePolicy) -> Self {
        use NewBacktracePolicy::*;
        match backtrace_policy {
            Default | Capture => { Self::capture() }
            NoBacktrace       => { Self::empty() }
            ForceCapture      => { Self::force_capture() }
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
            Default | InheritOrCapture  => { if source_bt.is_captured() { source_bt } else { Self::capture() } }
            InheritOrForceCapture       => { if source_bt.is_captured() { source_bt } else { Self::force_capture() } }
        }
    }

    pub fn is_captured(&self) -> bool { self.not_captured.is_some() || self.inner.is_none() }

    pub fn capture() -> Self {
        let bt = std::backtrace::Backtrace::capture();
        let not_captured_status: Option<NotCapturedInner> = std_backtrace_status_to_inner_not_captured(&bt);

        BacktraceInfo {
            not_captured: not_captured_status,
            inner: Some(BSRc::new(bt)),
        }
    }

    pub fn force_capture() -> Self {
        let bt = std::backtrace::Backtrace::force_capture();
        let not_captured_status: Option<NotCapturedInner> = std_backtrace_status_to_inner_not_captured(&bt);

        BacktraceInfo {
            not_captured: not_captured_status,
            inner: Some(BSRc::new(bt)),
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

    // TODO: Do not use it manually. Use something like: self.inherit(), self.inherit_or_capture()/self.new_or()
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
            ptr_backtrace.status()
        }
        else {
            std::backtrace::BacktraceStatus::Unsupported
        }
    }

    pub fn backtrace(&self) -> &std::backtrace::Backtrace {
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
    }
}


impl std::fmt::Debug for BacktraceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::backtrace::*;

        match self.backtrace_status() {
            BacktraceStatus::Unsupported => { write!(f, "Backtrace unsupported") }
            BacktraceStatus::Disabled    => { write!(f, "Backtrace disabled")    }
            BacktraceStatus::Captured    => {
                match self.inner {
                    Some(ref backtrace) => { write!(f, "\n{}", backtrace) }
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


pub fn enable_backtrace() {
    let to_enable_value = "1"; // or "full" ??
    std::env::set_var("RUST_BACKTRACE", to_enable_value);
}

pub fn disable_backtrace() {
    std::env::set_var("RUST_BACKTRACE", "0");
}
