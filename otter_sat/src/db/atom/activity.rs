use crate::{config::Activity, db::atom::AtomDB, structures::atom::Atom};

/// Methods for inspecting and mutating the activity of an atoms.
///
/// The role of these methods is tied to the use of [VSIDS](crate::config::vsids).
impl AtomDB {
    #[allow(non_snake_case)]
    /// Bumps the activities of each atom in the given iterator, and increases the bump for next time.
    ///
    /// If the bumped activity would be greater than the maximum allowed activity, the activity of every atom is rescored.
    pub fn bump_relative<A: Iterator<Item = Atom>>(&mut self, atoms: A) {
        for atom in atoms {
            if self.activity_of(atom) + self.config.bump.value > self.config.bump.max {
                self.rescore_activity()
            }
            self.bump_activity(atom);
        }

        self.exponent_activity();
    }

    /// Pops the most active atoms from the activity heap.
    pub fn heap_pop_most_active(&mut self) -> Option<Atom> {
        self.activity_heap.pop_max().map(|idx| idx as Atom)
    }

    /// The acitivty of an atom, regardless of whether it is on the activity heap.
    pub fn activity_of(&self, atom: Atom) -> Activity {
        *self.activity_heap.value_at(atom as usize)
    }

    /// Bumps the activity of an atom and updates it's position on the activity heap, if the atom is on the activity heap.
    pub fn bump_activity(&mut self, atom: Atom) {
        self.activity_heap.revalue(
            atom as usize,
            self.activity_of(atom) + self.config.bump.value,
        );
        self.activity_heap.heapify_if_active(atom as usize);
    }

    /// Increase the activity bump applied to atoms by a factor.
    pub fn exponent_activity(&mut self) {
        let factor = 1.0 / (1.0 - self.config.decay.value);
        self.config.bump.value *= factor
    }

    /// Rescores the activity of all atoms and the activity bump .
    pub fn rescore_activity(&mut self) {
        let heap_max = self
            .activity_heap
            .peek_max_value()
            .copied()
            .unwrap_or(Activity::MIN);
        let rescale = Activity::max(heap_max, self.config.bump.value);

        let factor = 1.0 / rescale;
        let rescale = |v: &Activity| v * factor;
        self.activity_heap.apply_to_all(rescale);
        self.config.bump.value *= factor;
        self.activity_heap.reheap();
    }
}
