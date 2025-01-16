use crate::structures::atom::Atom;
use crate::structures::valuation::Valuation;

use super::AtomDB;

/// Methods associated with the valuation stored in a [AtomDB].
///
/// # Safety
/// As relevant methods do not ensure an atom is present in the valuation before attempting to access stored information about the atom they inclued unsafe blocks.
///
/// Still, by construction …
impl AtomDB {
    /// Returns the value of the atom from the valuation stored in the [AtomDB].
    /// # Safety
    /// Does not check that the atom is part of the valuation.
    pub fn value_of(&self, atom: Atom) -> Option<bool> {
        unsafe { *self.valuation.get_unchecked(atom as usize) }
    }

    /// Returns the '*previous*' value of the atom from the valuation stored in the [AtomDB].
    ///
    /// When a context is built this value may be randomised.
    ///
    /// # Safety
    /// Does not check that the atom is part of the valuation.
    pub fn previous_value_of(&self, atom: Atom) -> bool {
        unsafe { *self.previous_valuation.get_unchecked(atom as usize) }
    }

    /// Clears the value of the atom from the valuation stored in the [AtomDB].
    /// # Safety
    /// Does not check that the atom is part of the valuation.
    pub fn clear_value(&mut self, atom: Atom) {
        if let Some(present) = self.value_of(atom) {
            unsafe { *self.previous_valuation.get_unchecked_mut(atom as usize) = present };
        }

        unsafe { *self.valuation.get_unchecked_mut(atom as usize) = None };
        unsafe { *self.decision_indicies.get_unchecked_mut(atom as usize) = None };
    }

    /// A string representing the current valuation, using the external representation of atoms.
    pub fn valuation_string(&self) -> String {
        self.valuation()
            .av_pairs()
            .filter_map(|(i, v)| {
                let idx = i as Atom;
                match v {
                    None => None,
                    Some(true) => Some(format!(" {}", self.external_representation(idx))),
                    Some(false) => Some(format!("-{}", self.external_representation(idx))),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// A string representing the current valuation, using the internal representation of atoms.
    pub fn internal_valuation_string(&self) -> String {
        let mut v = self
            .valuation()
            .av_pairs()
            .filter_map(|(i, v)| match v {
                None => None,
                Some(true) => Some(i as isize),
                Some(false) => Some(-(i as isize)),
            })
            .collect::<Vec<_>>();
        v.sort_unstable();
        v.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// A string representing the current valuation and the decision levels at which atoms were valued.
    /// The internal representation of atoms is used.
    pub fn internal_valuation_decision_string(&self) -> String {
        let mut v = self
            .valuation()
            .av_pairs()
            .filter_map(|(i, v)| match v {
                None => None,
                Some(true) => Some(i as isize),
                Some(false) => Some(-(i as isize)),
            })
            .collect::<Vec<_>>();
        v.sort_unstable();
        v.iter()
            .map(|v| unsafe {
                format!(
                    "{} ({:?})",
                    v,
                    self.decision_index_of(v.unsigned_abs() as Atom).unwrap()
                )
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}
