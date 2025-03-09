use crate::structures::atom::Atom;
use crate::structures::literal::IntLiteral;
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
    pub unsafe fn clear_value(&mut self, atom: Atom) {
        if let Some(present) = self.value_of(atom) {
            *self.previous_valuation.get_unchecked_mut(atom as usize) = present;
        }

        *self.valuation.get_unchecked_mut(atom as usize) = None;
        *self.decision_indicies.get_unchecked_mut(atom as usize) = None;
    }

    /// A string representing the current valuation, using the external representation of atoms.
    pub fn valuation_string(&self) -> String {
        self.valuation()
            .atom_value_pairs()
            .filter_map(|(atom, v)| match v {
                None => None,
                Some(true) => Some(format!(" {atom}")),
                Some(false) => Some(format!("-{atom}")),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// A string representing the current valuation, using [IntLiteral]s.
    pub fn valuations_ints(&self) -> Vec<IntLiteral> {
        self.valuation()
            .atom_value_pairs()
            .filter_map(|(atom, v)| match v {
                None => None,
                Some(true) => Some(atom as IntLiteral),
                Some(false) => Some(-(atom as IntLiteral)),
            })
            .collect()
    }

    /// A string representing the current valuation, using the internal representation of atoms.
    pub fn internal_valuation_string(&self) -> String {
        self.valuation()
            .atom_value_pairs()
            .filter_map(|(atom, v)| match v {
                None => None,
                Some(true) => Some((atom as isize).to_string()),
                Some(false) => Some((-(atom as isize)).to_string()),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// A string representing the current valuation and the decision levels at which atoms were valued.
    /// The internal representation of atoms is used.
    pub fn internal_valuation_decision_string(&self) -> String {
        unsafe {
            self.valuation()
                .atom_value_pairs()
                .filter_map(|(atom, v)| match self.level_unchecked(atom) {
                    None => None,
                    Some(level) => match v {
                        None => None,
                        Some(true) => Some(format!("{atom} ({level})",)),
                        Some(false) => Some(format!("-{atom} ({level})",)),
                    },
                })
                .collect::<Vec<_>>()
                .join(" ")
        }
    }
}
