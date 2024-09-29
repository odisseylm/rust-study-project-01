
// This file contains type aliases for third-party types visible outside.
// It is needed if other modules use the same third-party crates, but with different versions.
//

#[allow(unused_imports)]
pub mod backtrace {
    pub use mvv_common::backtrace::{backtrace, BacktraceCell};
}
