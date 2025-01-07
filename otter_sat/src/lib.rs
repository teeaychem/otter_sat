//! A library for determining the satisfiability of boolean formulas written in conjunctive normal form.
//!
//! otter_sat is a library for determining the satisfiability of boolean formulas written in conjunctive normal form, using a variety of techniques from the literature on conflict-driven clause-learning solving, and with support for incremental solves.
//!
//! otter_sat is developed to help researchers, developers, or anyone curious, to investigate satisfiability solvers, whether as a novice or through implementing novel ideas.
//! In this respect, otter_sat may (eventually) be seen as similar to [MiniSAT](minisat.se).
//!
//! Some guiding principles of otter_sat are (see [below](#guiding-principles) for further details):
//! - [Modularity](#modularity).
//! - Documentation, of both implementation and theory.
//! - [Verification](#verification).
//! - [Simple efficiency](#simple-efficiency).
//!
//! # Orientation
//!
//! The library is design around the core structure of a [context].
//!
//! Contexts are built with a configuration and optional method for recording [dispatches](crate::dispatch) from a solve, and clauses may be added though the [DIMACS](crate::context::GenericContext::read_dimacs) representation of a formula or [programatically](crate::context::GenericContext::clause_from_string).
//!
//! Internally, and at a high-level, a solve is viewed in terms of manipulation of, and relationships between, a handful of databases which instantiate core theoretical objects.
//! Notably:
//! - A formula is stored in a clause database.
//! - A valuation is stored in an atom database.
//! - Consequences of the current valuation with respect to the formula are stored in a literal database.
//!
//! Consequences follow a current valuation and formula, which in turn lead to a revised valuation and/or formula, from which further consequences follow.
//! And, in terms of implementation, data from the clause and atom database is read, used to update the literal database, which in turn leads to revisions of the atom and/or clause database.
//!
//! Useful starting points, then, may be:
//! - The high-level [solve procedure](crate::procedures::solve) to inspect the dynamics of a solve.
//! - The [database module](crate::db) to inspect the data considered during a solve.
//! - The [structures] to familiarise yourself with the abstract elements of a solve and their representation (formulas, clauses, etc.)
//! - The [configuration](crate::config) to see what features are supported.
//!
//! If you're in search of cnf formulas consider:
//! - The SATLIB benchmark problems at [www.cs.ubc.ca/~hoos/SATLIB/benchm.html](https://www.cs.ubc.ca/~hoos/SATLIB/benchm.html)
//! - The Global Benchmark Database at [benchmark-database.de](https://benchmark-database.de)
//! - SAT/SMT by Example at [smt.st](https://smt.st)
//!
//! # Examples
//!
//! + Find (a count of) all valuations of some collection of atoms.
//!
//! ```rust
//! # use otter_sat::config::Config;
//! # use otter_sat::context::Context;
//! # use otter_sat::dispatch::library::report::{self};
//! // setup a context to solve within.
//! let mut the_context: Context = Context::from_config(Config::default(), None);
//!
//! // Each character in the string is interpreted as an atom.
//! let atoms = "model";
//! for atom in atoms.chars() { // add atoms to the context.
//!     assert!(the_context.atom_from_string(&atom.to_string()).is_ok())
//! }
//!
//! let mut count = 0;
//!
//! loop {
//!     // Clear any decisions made on a previous solve and
//!     the_context.clear_decisions();
//!     // Determine the satisfiability of the formula in the context.
//!     assert!(the_context.solve().is_ok());
//!
//!     // Break from the loop as soon as the context is unsatisfiable.
//!     match the_context.report() {
//!         report::Solve::Satisfiable => {}
//!         _ => break,
//!     };
//!
//!     count += 1;
//!
//!     // Read the (satisfying) valuation from the present solve.
//!     let valuation = the_context.atom_db.valuation_string();
//!
//!     // Create the string representation of a clause to force a new valuation.
//!     let mut new_valuation = String::new();
//!     for literal in valuation.split_whitespace() {
//!         match literal.chars().next() {
//!             Some('-') => new_valuation.push_str(&literal[1..]),
//!             Some(_) => new_valuation.push_str(format!("-{literal}").as_str()),
//!             None => break,
//!         };
//!         new_valuation.push(' ');
//!     }
//!
//!     // Transform the string to a clause and add the clause to the solve.
//!     let the_clause = the_context.clause_from_string(&new_valuation).unwrap();
//!     match the_context.add_clause(the_clause) {
//!         Ok(()) => {}
//!         Err(_) => break,
//!     };
//! }
//! // Check the expected number of models were found.
//! assert_eq!(count, 2_usize.pow(atoms.len().try_into().unwrap()));
//! ```
//!
//! + Parse and solve a DIMACS formula.
//!
//! ```rust
//! # use otter_sat::context::Context;
//! # use otter_sat::config::Config;
//! # use std::io::Write;
//! # use otter_sat::dispatch::library::report::{self};
//! # use otter_sat::types::err::{self};
//! let mut the_context = Context::from_config(Config::default(), None);
//!
//! let mut dimacs = vec![];
//! let _ = dimacs.write(b"
//!  p  q 0
//! -p  q 0
//! -p -q 0
//!  p -q 0
//! ");
//!
//! the_context.read_dimacs(dimacs.as_slice());
//! the_context.solve();
//! assert_eq!(the_context.report(), report::Solve::Unsatisfiable);
//! ```
//!
//! + Identify unsatisfiability of a DIMACS formula during parsing.
//!
//! ```rust
//! # use otter_sat::context::Context;
//! # use otter_sat::config::Config;
//! # use std::io::Write;
//! # use otter_sat::dispatch::library::report::{self};
//! # use otter_sat::types::err::{self};
//! let mut the_context = Context::from_config(Config::default(), None);
//!
//! let mut dimacs = vec![];
//! let _ = dimacs.write(b"
//!  p       0
//! -p  q    0
//! -p -q  r 0
//!       -r 0
//! ");
//!
//! assert_eq!(the_context.read_dimacs(dimacs.as_slice()), Err(err::Build::Unsatisfiable));
//! ```
//!
//! # Guiding principles
//!
//! ## Modularity
//!
//!   + A solver is built of many interconnected parts, but where possible (and reasonable) interaction between parts happens through documented access points. For example:
//!     - Clauses are stored in a [clause database](db::clause), and are accesseed through [keys](db::ClauseKey).
//!       An internal distinction is made between unit clauses, binary clauses, and long(er) clauses.
//!       This distinction is encoded in the clause keys, and supports a variety of methods, but the internal structure of the clause database is private.
//!     - Things such as [literals](structures::literal) and [clauses](structures::clause) are defined first as traits whose canonical instantations are used only when there is 'good reason' to do so.
//!     - The algorithm for determining satisfiability is factored into a collection of [procedures].
//!     - Use of external crates is limited to crates which help support modularity, such as [log](https://docs.rs/log/latest/log/) and [rand](https://docs.rs/rand/latest/rand/).
//!
//! ## Verification
//!
//! + The core solver (excluding techniques such as subsumption) supports generation of [FRAT proofs](https://arxiv.org/pdf/2109.09665v1) which can be checked by independent tools such as [FRAT-rs](https://github.com/digama0/frat).
//!
//! + Verification itself is handled via a system for sending dispatches from a solve, and incurrs minimal overhead (checks on an optional) when not used.\
//!   As a consequence of this, the system for handling dispatches is somewhat complex.
//!   And, as a consequence of *that* an effort has been made to make ignoring the system easy.
//!
//! ## Simple efficiency
//!
//! The solver is efficient in most operations, and known inefficiencies are often noted.
//! Still, while comprimises are made for the same of efficiency, overall the library is written using mostly simple Rust, with annotated uses of unsafe, notes on when using a function would be unsound, and fights with the borrow checker explained.
//!   + The library makes free use of unsafe so long as a reason for why the relevant invariant is maintained.
//!   + Though, many relevant invariants escape the borrow checker, and for this purpose 'soundness' notes are made where relevant.
//!   + In addition, there are times when some not-so-simple Rust is required to appease the borrow checker (notably [BCP](crate::procedures::bcp)) and explanations are given of these.
//!
//! # Logs
//!
//! To help diagnose issues (somewhat) detailed calls to [log!](log) are made, and a variety of targets are defined in order to help narrow output to relevant parts of the library.
//! As logging is only built on request, and further can be requested by level, logs are verbose.
//!
//! The targets are lists in [misc::log].
//!
//! For example, when used with [env_logger](https://docs.rs/env_logger/latest/env_logger/):
//! - Logs related to [the clause database](crate::db::clause) can be filtered with `RUST_LOG=clause_db …` or,
//! - Logs of reduction count without information about the clauses removed can be found with `RUST_LOG=reduction=info …`
//!

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
