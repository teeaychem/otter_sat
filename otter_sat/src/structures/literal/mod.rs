//! Literals are atoms paired with a (boolean) polarity.
//!
//! Or, rather, anything which has methods for returning an atom and a polarity (and a few other useful things).
//!
//! The 'canonical' implementation of the literal trait is given by the [CLiteral] structure.
//! This is either:
//! - An [IntLiteral], which aliases a literal to an integer such that the absolute value of the integer is the atom of the literal, and the sign of the intger is the polarity of the literal.
//! - An [ABLiteral] which holds an atom (the 'A') and a boolean (the 'B') representing the polarity of the literal.
//!
//! <div class="warning">
//! Almost all interaction with literals in the library is through the canonical representation in order to the compiler to decide whether or not to borrow or take ownership.
//! </div>
//!
//! Implementation of the literal trait requires implementation of two additional traits:
//! - [Ord]
//!   + Literals should be ordered by atom and then polarity, with the (Rust default) ordering of 'false' being (strictly) less than 'true'.
//! - [Hash](std::hash::Hash)
//!   + Literals are hashable in order to allow for straightforward use of literals as indicies of maps, etc.
//!     This is particularly useful when recording information from [dispatches](crate::dispatch).
//!
//! # Examples
//!
//! ```rust
//! # use otter_sat::structures::literal::{CLiteral, Literal};
//! let atom = 79;
//! let polarity = true;
//! let literal = CLiteral::new(atom, polarity);
//!
//! assert!(literal.polarity());
//!
//! assert!(literal.atom().cmp(&79).is_eq());
//! assert!(literal.negate().polarity().cmp(&false).is_eq());
//!
//! assert!(literal.cmp(&CLiteral::new(79, !false)).is_eq());
//! ```
//!
//! Preference is given to abstracting from the specific implementation of literals by using the [CLiteral] type alias.
//! Still, if [ABLiteral]s or [IntLiteral]s are used, some traits have agnostic implementations.
//!
//! ```rust
//! # use otter_sat::structures::literal::{ABLiteral, CLiteral, IntLiteral, Literal};
//! let atom = 14;
//! let polarity = true;
//!
//! let canonical_literal = CLiteral::new(atom, polarity);
//! let ab_literal = ABLiteral::new(atom, polarity);
//! let int_literal = IntLiteral::new(atom, polarity);
//!
//! assert_eq!(ab_literal, int_literal);
//! assert_eq!(ab_literal, canonical_literal);
//! assert_eq!(canonical_literal, int_literal);
//! ```

#[allow(non_snake_case)]
#[doc(hidden)]
mod ab_literal;
pub use ab_literal::ABLiteral;

mod int_literal;
pub use int_literal::IntLiteral;

use crate::structures::atom::Atom;

/// Something which has methods for returning an atom and a polarity, etc.
pub trait Literal: std::cmp::Ord + std::hash::Hash {
    /// A fresh literal, specified by pairing an atom with a boolean.
    fn new(atom: Atom, polarity: bool) -> Self;

    /// The negation of the literal.
    fn negate(&self) -> Self;

    /// The atom of the literal.
    fn atom(&self) -> Atom;

    /// The polarity of the literal.
    fn polarity(&self) -> bool;

    /// The literal in it's 'canonical' form of an atom paired with a boolean.
    fn canonical(&self) -> CLiteral;

    /// The literal in it's integer form, with sign indicating polarity.
    fn as_int(&self) -> isize;
}

#[cfg(not(feature = "boolean"))]
/// The canonical implementation of a literal.
pub type CLiteral = IntLiteral;

#[cfg(feature = "boolean")]
/// The canonical implementation of a literal.
pub type CLiteral = ABLiteral;
