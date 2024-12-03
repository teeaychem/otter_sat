#![allow(unused_imports)]
//! A library for determining the satisfiability of boolean formulas written in conjunctive normal form.
//!
//!
//! # The context
//! The library is build around the core structure of a 'context'.
//!
//! - Context
//!     - The clause database \
//!       A collection of clauses, each indexed by a clause key. \
//!       From an external perspective there are two important kinds of clause:
//!       * Original clauses \
//!         Original clauses are added to the context from some external source (e.g. directly or through some DIMACS file). \
//!         The collection of original clauses together with the collection of original literals are the CNF formula 𝐅 whose satisfiability may be determined.
//!       * Added clauses \
//!         Clauses added to the context by some procedure (e.g. via resolution).
//!         Every added clause is a consequence of the collection of original clauses.
//!
//!     - The literal database \
//!       The literal database handled structures who primary
//!       * Proven literals
//!       * The choice stack
//!     - The variable database
//!       * Valuation
//!       * Watch database
//!   - Consequence queue
//!
//! # Design
//! - High-level parts are easy to compose
//! - Low-level parts are easy to extend or modify
//!
//! # Features
//! - Dispatches
//!   * [FRAT](crate::dispatch::frat)
//!   * [Unsatisfiable core](crate::dispatch::core)
//!
//! # A short guide to terminology (or, the variety of ways in which some common words are used)
//!
//! Value is true or false.
//! Or, '1' and '0', or 'is to be' and 'is not to be'.
//! Boolean, two values.
//!
//! Variable is some thing with a name to which assigning true or false is of interest.
//! In logic literature, the are often called 'atoms'.
//!
//! Language
//! Set of variables.
//! Language is implicitly given by relevant variables.
//!
//! Valuation
//! Assigment of a value to each variable in the language
//!
//! Partial valuation
//! Assignment of value to a set of variables in the language.
//!
//! Literal is a pair of a literal and a value.
//!
//! Clause in the literature is a set of literals.
//! The empty clause, 'singleton' or 'unit' clause, 'binary' clause.
//!
//! Formula
//! Collection of clauses
//!
//! Variable as a strucure
//! A 32-bit unsigned integer.
//!
//! Clauses as a structure.
//! Always contain at least two literals.
//! Unit clauses are identified with contained literal.
//!
//! Literal as a structure
//! A structure containig a literal and a value.
//! Often implemented as an integer, with value given by the sign of the integer, no sign true, sign false.
//! 'Minus as negation', and is used for input.
//!
//! Resolution
//!
//! Private items are documented.

#![allow(mixed_script_confusables)]
#![allow(unused_must_use)]
#![allow(clippy::single_match)]
#![allow(clippy::collapsible_else_if)]
// #![allow(unused_imports)]

#[doc(hidden)]
mod builder;
#[doc(hidden)]
mod procedures;

pub mod config;
pub mod context;
pub mod structures;
pub mod types;

pub mod generic;

pub mod dispatch;

pub mod db;

pub mod misc;
pub mod transient;
