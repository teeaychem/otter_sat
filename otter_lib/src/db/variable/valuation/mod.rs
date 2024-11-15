use crate::structures::variable::Variable;

use super::VariableDB;

mod context;

impl VariableDB {
    pub fn value_of(&self, v_idx: Variable) -> Option<bool> {
        unsafe { *self.valuation.get_unchecked(v_idx as usize) }
    }

    pub fn previous_value_of(&self, v_idx: Variable) -> bool {
        unsafe { *self.previous_valuation.get_unchecked(v_idx as usize) }
    }

    pub(super) fn clear_value(&mut self, v_idx: Variable) {
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
