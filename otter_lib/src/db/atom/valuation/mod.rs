use crate::structures::atom::Atom;

use super::AtomDB;

mod context;

impl AtomDB {
    pub fn value_of(&self, v_idx: Atom) -> Option<bool> {
        unsafe { *self.valuation.get_unchecked(v_idx as usize) }
    }

    pub fn previous_value_of(&self, v_idx: Atom) -> bool {
        unsafe { *self.previous_valuation.get_unchecked(v_idx as usize) }
    }

    pub(super) fn clear_value(&mut self, v_idx: Atom) {
        if let Some(present) = self.value_of(v_idx) {
            unsafe {
                *self.previous_valuation.get_unchecked_mut(v_idx as usize) = present;
            }
        }
        unsafe {
            *self.valuation.get_unchecked_mut(v_idx as usize) = None;
            *self.choice_indicies.get_unchecked_mut(v_idx as usize) = None;
        }
    }
}
