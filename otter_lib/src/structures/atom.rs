//! (The internal representation of) an atom.
//!
//! Each atom is a u32 *u* such that either:
//! - *u* is 0, or:
//! - *u - 1* is an atom.
//!
//! ```rust
//! # use otter_lib::structures::atom::Atom;
//! let m = 97;
//! let atoms = (0..m).collect::<Vec<Atom>>();
//!
//! let atom = 97;
//! ```
//!
//!
//! That the atoms are [0..*m*) for some *m*.
//!
//! This representation allows atoms to be used as the indicies of a structure, e.g. `exteranal_string[a]` without taking too much space.
//! Revising the representation to any unsigned integer is possible.

/// An atom, aka. a 'variable'.
pub type Atom = u32;
