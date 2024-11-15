use crate::{
    config::{Activity, Config},
    db::variable::VariableDB,
    structures::variable::Variable,
};

impl VariableDB {
    #[allow(non_snake_case)]
    /// Bumps the activities of each variable in 'variables'
    /// If given a hint to the max activity the rescore check is performed once on the hint
    pub fn apply_VSIDS<V: Iterator<Item = Variable>>(&mut self, variables: V, config: &Config) {
        for variable in variables {
            if self.activity_of(variable as usize) + config.activity_conflict > config.activity_max
            {
                self.rescore_activity()
            }
            self.bump_activity(variable as usize);
        }

        self.exponent_activity(config);
    }

    pub fn heap_pop_most_active(&mut self) -> Option<Variable> {
        self.activity_heap.pop_max().map(|idx| idx as Variable)
    }
}

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
