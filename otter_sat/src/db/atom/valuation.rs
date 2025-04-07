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

    /// Returns the '*previous*' value of the atom from the valuation stored in the [AtomDB].
    ///
    /// When a context is built this value may be randomised.
    ///
    /// # Safety
    /// Does not check that the atom is part of the valuation.
    pub fn previous_value_of(&self, atom: Atom) -> bool {
        unsafe { *self.previous_valuation.get_unchecked(atom as usize) }
    }
}
