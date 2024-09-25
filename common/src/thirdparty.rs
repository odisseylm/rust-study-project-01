
// This file contains type aliases for third-party types visible outside.
// It is needed if other modules use the same third-party crates, but with different versions.
//


pub mod smallvec {
    pub use smallvec::Array;
    pub use smallvec::SmallVec;
}
