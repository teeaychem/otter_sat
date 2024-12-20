use crate::structures::{atom::Atom, valuation::Valuation};

impl<T: std::ops::Deref<Target = [Option<bool>]>> Valuation for T {
    fn value_of(&self, atom: Atom) -> Option<Option<bool>> {
        self.get(atom as usize).copied()
    }

    unsafe fn unchecked_value_of(&self, atom: Atom) -> Option<bool> {
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
