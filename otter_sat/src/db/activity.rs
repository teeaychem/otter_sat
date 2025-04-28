use crate::{
    config::{Activity, ConfigOption},
    db::{clause::activity_glue::ActivityLBD, keys::FormulaIndex},
    generic::index_heap::IndexHeap,
    structures::atom::Atom,
};

use super::clause::ClauseDB;

/// Bumps the activities of each atom in the given iterator, and increases the bump for next time.
///
/// If the bumped activity would be greater than the maximum allowed activity, the activity of every atom is rescored.
pub fn bump_atoms_relative<Atoms: Iterator<Item = Atom>>(
    atoms: Atoms,
    index_heap: &mut IndexHeap<Activity>,
    bumpy: &mut ConfigOption<Activity>,
    decay: &mut ConfigOption<Activity>,
) {
    for atom in atoms {
        // Rescore the activity of all atoms and the activity bump, if needed.
        if *index_heap.value_at(atom as usize) + bumpy.value > bumpy.max {
            let heap_max = index_heap
                .peek_max_value()
                .copied()
                .unwrap_or(Activity::MIN);

            let factor = 1.0 / Activity::max(heap_max, bumpy.value);
            bumpy.value *= factor;
            let rescale = |v: &Activity| v * factor;
            index_heap.apply_to_all(rescale);
            index_heap.heapify();
        }

        // Bump the activity of an atom and updates it's position on the activity heap, if the atom is on the activity heap.
        index_heap.revalue(
            atom as usize,
            *index_heap.value_at(atom as usize) + bumpy.value,
        );
        index_heap.heapify_if_active(atom as usize);
    }

    // Increase the activity bump applied to atoms by a factor.
    bumpy.value *= 1.0 / (1.0 - decay.value);
}

impl ClauseDB {
    /// Bumps the activity of a clause, rescoring all acitivies if needed.
    ///
    /// See the corresponding method with respect to atoms for more details.
    pub fn bump_activity(&mut self, index: FormulaIndex) {
        if let Some(max) = self.activity_heap.peek_max_value() {
            if max.activity + self.config.bump.value > self.config.bump.max {
                let factor = 1.0 / max.activity;
                let decay_activity = |s: &ActivityLBD| ActivityLBD {
                    activity: s.activity * factor,
                    lbd: s.lbd,
                };
                self.activity_heap.apply_to_all(decay_activity);
                self.config.bump.value *= factor
            }
        }

        let bump_activity = |s: &ActivityLBD| ActivityLBD {
            activity: s.activity + self.config.bump.value,
            lbd: s.lbd,
        };

        let index = index as usize;
        self.activity_heap
            .apply_to_value_at_value_index(index, bump_activity);
        self.activity_heap.heapify_if_active(index);

        self.config.bump.value *= 1.0 / (1.0 - self.config.decay.value);
    }
}
