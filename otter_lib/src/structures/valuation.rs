//! Values!

use super::atom::Atom;

/// The default representation of a valuation.
pub type ValuationV = Vec<Option<bool>>;

/// A valuation is something which stores some value of a atom and/or perhaps the information that the atom has no value.
pub trait Valuation {
    /// Some value of a atom under the valuation, or otherwise nothing.
    /// # Safety
    /// Implementations of `value_of` are not required to check the atom is part of the valuation.
    unsafe fn value_of(&self, atom: Atom) -> Option<bool>;

    /// An iterator over the values of a atoms in the valuation, in strict, continguous, atom order.
    /// I.e. the first element is the atom '1' and then *n*th element is atom *n*.
    fn values(&self) -> impl Iterator<Item = Option<bool>>;

    /// An iterator through all (Atom, Value) pairs.
    fn vv_pairs(&self) -> impl Iterator<Item = (Atom, Option<bool>)>;

    /// An iterator through atoms which have some value.
    fn valued_atoms(&self) -> impl Iterator<Item = Atom>;

    /// An iterator through atoms which do not have some value.
    fn unvalued_atoms(&self) -> impl Iterator<Item = Atom>;
}

impl<T: std::ops::Deref<Target = [Option<bool>]>> Valuation for T {
    unsafe fn value_of(&self, atom: Atom) -> Option<bool> {
        *self.get_unchecked(atom as usize)
    }

    fn values(&self) -> impl Iterator<Item = Option<bool>> {
        self.iter().copied()
    }

    fn vv_pairs(&self) -> impl Iterator<Item = (Atom, Option<bool>)> {
        self.iter()
            .enumerate()
            .map(|(var, val)| (var as Atom, *val))
    }

    fn valued_atoms(&self) -> impl Iterator<Item = Atom> {
        self.iter().enumerate().filter_map(|(var, val)| match val {
            None => None,
            _ => Some(var as Atom),
        })
    }

    fn unvalued_atoms(&self) -> impl Iterator<Item = Atom> {
        self.iter().enumerate().filter_map(|(var, val)| match val {
            None => Some(var as Atom),
            _ => None,
        })
    }
}
