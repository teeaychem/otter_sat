use crate::structures::{
    atom::Atom,
    consequence::AssignmentSource,
    valuation::{CValuation, Valuation},
};

use super::AtomCells;

impl Valuation for AtomCells {
    fn value_of(&self, atom: Atom) -> Option<Option<bool>> {
        self.cells.get(atom as usize).map(|cell| cell.value)
    }

    unsafe fn value_of_unchecked(&self, atom: Atom) -> Option<bool> {
        unsafe { self.cells.get_unchecked(atom as usize).value }
    }

    fn values(&self) -> impl Iterator<Item = Option<bool>> {
        self.cells.iter().map(|cell| cell.value)
    }

    fn atom_value_pairs(&self) -> impl Iterator<Item = (Atom, Option<bool>)> {
        self.cells
            .iter()
            .enumerate()
            .skip(1)
            .map(|(var, cell)| (var as Atom, cell.value))
    }

    fn atom_valued_pairs(&self) -> impl Iterator<Item = (Atom, bool)> {
        self.cells
            .iter()
            .enumerate()
            .skip(1)
            .flat_map(|(atom, cell)| cell.value.map(|v| (atom as Atom, v)))
    }

    fn valued_atoms(&self) -> impl Iterator<Item = Atom> {
        self.cells
            .iter()
            .enumerate()
            .flat_map(|(atom, cell)| cell.value.map(|_| atom as Atom))
    }

    fn unvalued_atoms(&self) -> impl Iterator<Item = Atom> {
        self.cells
            .iter()
            .enumerate()
            .flat_map(|(atom, cell)| match cell.value {
                None => Some(atom as Atom),
                Some(_) => None,
            })
    }

    fn canonical(&self) -> CValuation {
        self.cells.iter().map(|cell| cell.value).collect::<Vec<_>>()
    }

    unsafe fn clear_value_of(&mut self, atom: Atom) {
        let cell = unsafe { self.cells.get_unchecked_mut(atom as usize) };
        cell.value = None;
        cell.source = AssignmentSource::None;
    }

    fn true_check(&self) -> bool {
        self.cells
            .first()
            .is_some_and(|cell| cell.value == Some(true))
    }

    fn atom_count(&self) -> usize {
        self.cells.len()
    }
}
