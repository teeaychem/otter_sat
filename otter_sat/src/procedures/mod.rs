//! Various procedures for mutating a context.
//!
//!For the most part these are methods accessed via a context, and primarily placed here for documentation.

pub mod analysis;
pub mod apply_consequences;
pub mod backjump;
pub mod bcp;
pub mod core;
pub mod decision;
pub mod schedulers;
pub mod solve;

#[doc(hidden)]
pub mod refresh;
