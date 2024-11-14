use crate::{
    config::{Activity, Config},
    db::variable::VariableDB,
    structures::variable::Variable,
};

impl VariableDB {
    pub(super) fn activity_of(&self, index: usize) -> Activity {
        *self.activity_heap.value_at(index)
    }

    pub(super) fn bump_activity(&mut self, index: usize) {
        self.activity_heap
            .update_one(index, self.activity_of(index) + self.score_increment)
    }

    pub(super) fn exponent_activity(&mut self, config: &Config) {
        let decay = config.variable_decay * 1e-3;
        let factor = 1.0 / (1.0 - decay);
        self.score_increment *= factor
    }

    pub(super) fn activity_max(&self) -> Option<Activity> {
        self.activity_heap.peek_max_value().copied()
    }

    pub(super) fn rescore_activity(&mut self) {
        let heap_max = self.activity_max().unwrap_or(Activity::MIN);
        let rescale = Activity::max(heap_max, self.score_increment);

        let factor = 1.0 / rescale;
        let rescale = |v: &Activity| v * factor;
        self.activity_heap.apply_to_all(rescale);
        self.score_increment *= factor;
        self.activity_heap.reheap();
    }
}

impl VariableDB {
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
