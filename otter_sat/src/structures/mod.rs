//! Key structures, such as literals and clauses.
//!
//! Most structures are made of a trait to capture the key features of the structure and a 'canonical' implementation of the trait.
//! Use of a trait or it's canonical implementation within the library is situational.
//!
//! # Other structures without a trait and/or canonical implementation.
//!
//! ## Formulas
//!
//!  A formula 𝐅 is set of [clauses](clause), interpreted as the conjunction of those clauses (and so is the conjunction of disjunctions over literals in some language).
//!
//!  The conjunction of clauses in the [clause database](crate::db::clause) is a *formula*, which is always entailed by the formula given to the context --- though the two formulas may differ due to preprocessing or the addition of entailed clauses, etc..
//!
//! ## Languages
//! A *language* 𝓛 is some set of [atoms](atom), closed under the operations of negation, conjunction, and disjunction. \
//! Every formula is expressed in some language, and every [context](crate::context) is implicity relative to some language.
//!
//! Languages do not have an implementation. \
//! Instead use context to determine which set of atoms constitutes *the* language of interest (typically, those atoms appearing in the input formula) and whether some collection of atoms belongs to the language, and if so whether it is negated, a conjunction, or disjunction.
//!
//! ## (Boolean) values
//!
//! A (boolean) values is one of two things.
//! Typically the first of the pair is identified as [true] and the second as [false]. \
//! Other choices include: 1 and 0, ⟙ (top) and ⟘ (bot), 'is to be' and 'is not to be', etc, but these are more difficult to implement.

pub mod atom;
pub mod clause;
pub mod literal;
pub mod valuation;
