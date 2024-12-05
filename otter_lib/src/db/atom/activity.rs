use crate::{config::Activity, db::atom::AtomDB, structures::atom::Atom};

impl AtomDB {
    #[allow(non_snake_case)]
    /// Bumps the activities of each atom in 'atoms'
    /// If given a hint to the max activity the rescore check is performed once on the hint
    pub fn apply_VSIDS<A: Iterator<Item = Atom>>(&mut self, atoms: A) {
        for atom in atoms {
            if self.activity_of(atom as usize) + self.config.bump > self.config.max_bump {
                self.rescore_activity()
            }
            self.bump_activity(atom as usize);
        }

        self.exponent_activity();
    }

    pub fn heap_pop_most_active(&mut self) -> Option<Atom> {
        self.activity_heap.pop_max().map(|idx| idx as Atom)
    }
}

impl AtomDB {
    pub(super) fn activity_of(&self, index: usize) -> Activity {
        *self.activity_heap.value_at(index)
    }

    pub(super) fn bump_activity(&mut self, index: usize) {
        self.activity_heap
            .revalue(index, self.activity_of(index) + self.config.bump);
        self.activity_heap.heapify_if_active(index);
    }

    pub(super) fn exponent_activity(&mut self) {
        let factor = 1.0 / (1.0 - self.config.decay);
        self.config.bump *= factor
    }

    pub(super) fn activity_max(&self) -> Option<Activity> {
        self.activity_heap.peek_max_value().copied()
    }

    pub(super) fn rescore_activity(&mut self) {
        let heap_max = self.activity_max().unwrap_or(Activity::MIN);
        let rescale = Activity::max(heap_max, self.config.bump);

        let factor = 1.0 / rescale;
        let rescale = |v: &Activity| v * factor;
        self.activity_heap.apply_to_all(rescale);
        self.config.bump *= factor;
        self.activity_heap.reheap();
    }
}
