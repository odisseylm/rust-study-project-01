

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


pub fn print_current_stack_trace() {
    let stacktrace = std::backtrace::Backtrace::capture();

    println!("{}", stacktrace);

    // this has bad formed one-line output
    // println!("{:?}", stacktrace);
}



pub struct BacktraceInfo {
    backtrace_status: std::backtrace::BacktraceStatus,
    backtrace: std::backtrace::Backtrace,
}


impl BacktraceInfo {
    pub fn new() -> Self {
        BacktraceInfo {
            backtrace_status: std::backtrace::BacktraceStatus::Captured,
            backtrace: std::backtrace::Backtrace::capture(),
        }
    }

    // We cannot return enum copy there since this enum is 'non_exhaustive'
    // and does not support 'copy/clone'.
    pub fn backtrace_status(&self) -> &std::backtrace::BacktraceStatus {
        &self.backtrace_status
    }
    pub fn backtrace(&self) -> &std::backtrace::Backtrace {
        &self.backtrace
    }
}


impl std::fmt::Debug for BacktraceInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use std::backtrace::*;

        match self.backtrace_status {
            BacktraceStatus::Unsupported => { write!(f, "Backtrace unsupported") }
            BacktraceStatus::Disabled    => { write!(f, "Backtrace disabled")    }
            BacktraceStatus::Captured    => { write!(f, "\n{}", self.backtrace)  }
            _ => { write!(f, "Unknown backtrace status.") }
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
