//! A (partial) function from atoms to truth values.
//!
//! If all atoms are assigned a value the valuation is 'full', otherwise the valuation is 'partial'.
//!
//! The canonical representation of a valuation as a vector of optional booleans, where each index of the vector is interpreted as an atom, though most interaction is through the valuation trait.
//!
//!  In other words, the canonical representation of a valuation 𝐯 is a vector *v* whose length is the number of atoms in the context such that:[^pedantic]
//!  -  *v*\[a\] = Some(true) *if any only if* 𝐯(𝐚) = true.
//!  -  *v*\[a\] = Some(false) *if any only if* 𝐯(𝐚) = false.
//!  -  *v*\[a\] = None *if any only if* 𝐯(𝐚) is undefined.
//!
//! The trait is implemented for anything which can be dereferenced to a slice of optional booleans.
//!
//! ```rust
//! # use otter_lib::structures::atom;
//! # use crate::otter_lib::structures::valuation::Valuation;
//! let atoms =     vec![0,    1,          2];
//! let valuation = vec![None, Some(true), None];
//!
//! assert_eq!(unsafe { valuation.unchecked_value_of(0) }, None);
//! assert_eq!(valuation.unvalued_atoms().count(), 2);
//! ```
//!
//! Throughout the library the unsafe `unchecked_value_of` is preferred over the safe `value_of`. \
//! This is because the implementation on vectors 'only' guarantees *memory* safety, while use requires the stronger guarantee that the (optional) value atom of interest is mapped to the index of the atom in the valuation, and with this an additional check that the atom really is there is redundant.
//!
//! [^pedantic]: Where 'a' is the internal representation of some atom whose external representation is '𝐚'.
//!
//! # Soundness
//!
//! The valuation trait is implemented for any structure which can be dereferenced to a slice of optional booleans.
//! And, as the value of an atom is determined by using the atom as an index on the dereferenced structure, there is no structural guarantee that the returned value is for the atom.
//!
//! In other words, the following is possible, unsound, and by (design/luck) fails:
//!
//! ```rust,should_panic
//! # use otter_lib::structures::atom;
//! # use crate::otter_lib::structures::valuation::Valuation;
//! let atoms =     vec![0,    1,          2];
//! let valuation = vec![None, Some(true), None];
//!
//! let sub_valuation = &valuation[1..];
//! assert_eq!(sub_valuation.value_of(0), Some(None));
//! ```

mod slice_impl;

use super::atom::Atom;

/// The canonical representation of a valuation.
#[allow(non_camel_case_types)]
pub type vValuation = Vec<Option<bool>>;

/// A valuation is something which stores some value of a atom and/or perhaps the information that the atom has no value.
pub trait Valuation {
    /// Some value of a atom under the valuation, or otherwise nothing.
    fn value_of(&self, atom: Atom) -> Option<Option<bool>>;

    /// Some value of a atom under the valuation, or otherwise nothing.
    /// # Safety
    /// Implementations are not required to check the atom is part of the valuation.
    unsafe fn unchecked_value_of(&self, atom: Atom) -> Option<bool>;

    /// An iterator over the values of a atoms in the valuation, in strict, continguous, atom order.
    /// I.e. the first element is the atom '1' and then *n*th element is atom *n*.
    fn values(&self) -> impl Iterator<Item = Option<bool>>;

    /// An iterator through all (Atom, Value) pairs.
    fn av_pairs(&self) -> impl Iterator<Item = (Atom, Option<bool>)>;

    /// An iterator through atoms which have some value.
    fn valued_atoms(&self) -> impl Iterator<Item = Atom>;

    /// An iterator through atoms which do not have some value.
    fn unvalued_atoms(&self) -> impl Iterator<Item = Atom>;
}
