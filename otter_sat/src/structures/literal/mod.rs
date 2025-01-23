//! Literals are atoms paired with a (boolean) polarity.
//!
//! Or, rather, anything which has methods for returning an atom and a polarity (and a few other useful things).
//!
//! The 'canonical' implementation of the literal trait is the [abLiteral] structure, made of an atom (the 'a') and a boolean (the 'b').
//! <div class="warning">
//! Almost all interaction with literals in the library is through the canonical (abLiteral) representation in order to the compiler to decide whether or not to borrow or take ownership of a literal.
//! </div>
//!
//! An example:
//!
//! ```rust
//! # use otter_sat::structures::literal::abLiteral;
//! # use crate::otter_sat::structures::literal::Literal;
//! let atom = 79;
//! let polarity = true;
//! let literal = abLiteral::fresh(atom, polarity);
//!
//! assert!(literal.polarity());
//!
//! assert!(literal.atom().cmp(&79).is_eq());
//! assert!(literal.negate().polarity().cmp(&false).is_eq());
//!
//! assert!(literal.cmp(&abLiteral::fresh(79, !false)).is_eq());
//! ```
//!
//! Implementation of the literal trait requires implementation of two additional traits:
//! - [Ord]
//!   + Literals should be ordered by atom and then polarity, with the (Rust default) ordering of 'false' being (strictly) less than 'true'.
//! - [Hash](std::hash::Hash)
//!   + Literals are hashable in order to allow for straightforward use of literals as indicies of maps, etc.
//!     This is particularly useful when recording information from [dispatches](crate::dispatch).
//!
//! In other solvers an integer is often used, with the sign of the integer indicating the value of the literal.

#[allow(non_snake_case)]
#[doc(hidden)]
mod impl_abLiteral;

use crate::structures::atom::Atom;

/// Something which has methods for returning an atom and a polarity, etc.
pub trait Literal: std::cmp::Ord + std::hash::Hash {
    /// A fresh literal, specified by pairing an atom with a boolean.
    fn fresh(atom: Atom, polarity: bool) -> Self;

    /// The negation of the literal.
    fn negate(&self) -> Self;

    /// The atom of the literal.
    fn atom(&self) -> Atom;

    /// The polarity of the literal.
    fn polarity(&self) -> bool;

    /// The literal in it's 'canonical' form of an atom paired with a boolean.
    fn canonical(&self) -> abLiteral;

    /// The literal in it's integer form, with sign indicating polarity.
    fn as_int(&self) -> isize;
}

/// The 'canonical' representation of a literal as an atom paired with a boolean.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy)]
pub struct abLiteral {
    /// The atom of a literal.
    atom: Atom,

    /// The polarity of a literal.
    polarity: bool,
}
