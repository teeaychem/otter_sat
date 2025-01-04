//! A library for determining the satisfiability of boolean formulas written in conjunctive normal form.
//!
//!
//! # The context
//! The library is build around the core structure of a '[context]'.
//!
//!   - [The clause database](crate::db::clause)
//!     + A collection of clauses, each indexed by a clause key. \
//!       From an external perspective there are two important kinds of clause:
//!       * Original clauses \
//!         Original clauses are added to the context from some external source (e.g. directly or through some DIMACS file). \
//!         The collection of original clauses together with the collection of original literals are the CNF formula 𝐅 whose satisfiability may be determined.
//!       * Added clauses \
//!         Clauses added to the context by some procedure (e.g. via resolution).
//!         Every added clause is a consequence of the collection of original clauses.
//!
//!   - [The literal database](crate::db::literal)
//!     + The literal database handled structures who primary
//!       * The choice stack
//!   - [The atom database](crate::db::atom)
//!     + Properties of atoms.
//!       * Valuation
//!       * Watch database
//! - [Consequence queue](crate::db::consequence_q)
//!
//! # Design
//! - High-level parts are easy to compose
//! - Low-level parts are easy to extend or modify
//!
//! # Features
//! - Dispatches
//!   * [FRAT](crate::dispatch::frat)
//!   * [Unsatisfiable core](crate::dispatch::core)

#![allow(mixed_script_confusables)]
#![allow(unused_must_use)]
#![allow(clippy::single_match)]
#![allow(clippy::collapsible_else_if)]
#![allow(clippy::derivable_impls)]
// #![allow(unused_imports)]

#[doc(hidden)]
pub mod builder;
pub mod procedures;

pub mod config;
pub mod context;
pub mod structures;
pub mod types;

pub mod generic;

pub mod dispatch;

pub mod db;

pub mod misc;
pub mod transient;

pub mod preprocessing;
