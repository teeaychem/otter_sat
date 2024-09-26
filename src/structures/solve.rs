pub mod core;
mod analysis;
mod mutation;
mod solves;
mod config;

pub use crate::structures::solve::core::{Solve, SolveError, SolveOk, SolveStatus};
pub use crate::structures::solve::config::SolveConfig;
pub use crate::structures::solve::solves::SolveResult;
