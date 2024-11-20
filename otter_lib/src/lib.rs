//! A library for determining the satisfiability of boolean formulas written in conjunctive normal form
//!
//! Design
//! - High-level parts are easy to compose
//! - Low-level parts are easy to modify
//!
//! Context
//! - Databases, linked
//! - Procedures
//! - Configuration
//! - Transient
//!
//! Dispatches
//! - FRAT
//! - Unsatisfiable core
//!
//! Private items are documented.

#![allow(mixed_script_confusables)]
#![allow(unused_must_use)]
#![allow(clippy::single_match)]
#![allow(clippy::collapsible_else_if)]
// #![allow(unused_imports)]

mod builder;
pub mod config;
pub mod context;
pub mod structures;
pub mod types;

mod procedures;

pub mod generic;

pub mod dispatch;

pub mod db;
mod misc;
mod transient;
