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
//!     - The atom database
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
//! - (Boolean) values.
//!   - A pair of things, with the first of the pair identified as `true` and the second as `false`.
//!     - Other choices include: `〈1, 0〉`, `〈⟙, ⟘〉`, `〈'is to be', 'is not to be'〉`.
//!   - Implementation:
//!     - The rust keyword [true](https://doc.rust-lang.org/std/keyword.true.html) is identified as `true` and the keyword [false](https://doc.rust-lang.org/std/keyword.false.html) is identified as `false`.
//!
//! - Atoms (aka. variables).
//!   - Things with a name to which assigning a (boolean) value (true or false) is of interest.
//!     - In the SAT literature these are often called 'variables' while in the logic literature these are often called 'atoms'.
//!   - Implentation:
//!     - A distinction is made between 'external' and 'internal' atoms.
//!       - External atoms
//!         - Are used to used during external interaction with a context, e.g. when providing a formula as input or reading the value of an atom. \
//!           External atoms are a string of non-whitespace characters that which does not being with '-' (a minus sign). \
//!           Examples: `p`, `atom_one`, `96`, `0`.
//!       - Internal atoms
//!         - Are used internal to a context.
//!         - Implementation:
//!           - A u32 `u` such that either (a) `u` is `0` or (b) `u - 1` is an atom --- i.e. the atoms belong to `[0..m)` for some `m`.
//!           - This representation allows atoms to be used as the indicies of a structure, e.g. `exteranal_string[a]` without taking too much space.
//!             Revising the representation to any unsigned integer is possible.
//!
//! - A language 𝓛.
//!   - A language is some set of atoms, closed under the operations of negation, conjunction, and disjunction.
//!   - Every formula is expressed in some language, and every context implicity uses a language.
//!     We do not specifically implement the representation of a language, and instead use context to determine which set of atoms constitutes *the* language of interest (typically, those atoms appearing in the input formula) and whether some collection of atoms belongs to the language, and if so whether it is negated, a conjunction, or disjunction.
//!
//!
//! - A (partial or full) valuation.
//!   - A function 𝐯 from the a language 𝓛 to the value true, the value false, or to 'no value'.
//!   - If some atom is assigned 'no value', the valuation is 'partial', otherwise the valuation is 'full'.
//!   - Implementation: \
//!       A vector `v` whose length is the number of atoms in 𝓛 such that: \
//!       `v[a] = Some(true)` if any only if 𝐯(a) = true \
//!       `v[a] = Some(false)` if any only if 𝐯(a) = false \
//!       `v[a] = None` if any only if 𝐯(a) = 'no value' \
//!        where `a` is the internal representation of some atom whose external representation is 'a'.
//!
//!
//!
//! - Literals.
//!   - Some pair of an atom and a value. \
//!     Often understood either an atom or the negation of an atom (especially in the logic literature).
//!     Though, as the pairing is so often made when the paired element is not intended as part of the langauge, and as intent is almost always clear by context the generalisation turns out to be quite useful.
//!   - Prefixing an atom with a '-' (minus sign) allows for input of a negated atom, and the same representation is used for output from the context --- e.g. `-p`, `-atom_one`, `-97`, etc.
//!   - Implentation: \
//!     Both as a trait `Literal`, and as a concrete structure implementing the trait. \
//!     The canonical structure implementing the traint is an `avLiteral` containing a an atom and a value.
//!     In other solvers an integer is often used, with the sign of the integer indicating the value of the literal.
//!
//! - Clauses.
//!   - Clauses are sets of literals, corresponding to the disjunction of the literals in the set (in the contextually relevant language), with the special case of the empty set being some expression in the langauge which is never true (e.g. 'p and not p').
//!   - Unit clauses (clauses containig a single literal) are often identified with the literal they contain.
//!   - Implementation: \
//!     Both as a trait `Clause`, and as a concrete structure implementing the trait. \
//!     The canonical structure implementing the traint is vector of literals (a `vClause`), and anything which may be dereferences o a slice of literals implents the trait, along with a lone literal.
//!
//! - Formula 𝐅
//!   - A set of clauses, interpreted as the conjunction of those clauses (and so is the conjunction of disjunctions over literals in some language).
//!   - Implementation:
//!     The clause database constitutes a formula, which is always entailed by the formula given to the context, though may differ due to preprocessing or learnt clauses.
//!
//! Private items are documented.
//!

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
