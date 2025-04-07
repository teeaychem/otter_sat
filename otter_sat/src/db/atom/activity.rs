use crate::{
    config::{Activity, ConfigOption},
    generic::index_heap::IndexHeap,
    structures::atom::Atom,
};

#[allow(non_snake_case)]
/// Bumps the activities of each atom in the given iterator, and increases the bump for next time.
///
/// If the bumped activity would be greater than the maximum allowed activity, the activity of every atom is rescored.
pub fn bump_relative<A: Iterator<Item = Atom>>(
    atoms: A,
    index_heap: &mut IndexHeap<Activity>,
    bumpy: &mut ConfigOption<Activity>,
    decay: &mut ConfigOption<Activity>,
) {
    for atom in atoms {
        if *index_heap.value_at(atom as usize) + bumpy.value > bumpy.max {
            rescore_activity(index_heap, bumpy);
        }
        bump_activity(atom, index_heap, bumpy);
    }

    exponent_activity(bumpy, decay);
}

/// Rescores the activity of all atoms and the activity bump.
fn rescore_activity(index_heap: &mut IndexHeap<Activity>, bumpy: &mut ConfigOption<Activity>) {
    let heap_max = index_heap
        .peek_max_value()
        .copied()
        .unwrap_or(Activity::MIN);
    let rescale = Activity::max(heap_max, bumpy.value);

    let factor = 1.0 / rescale;
    let rescale = |v: &Activity| v * factor;
    index_heap.apply_to_all(rescale);
    bumpy.value *= factor;
    index_heap.heapify();
}

/// Bumps the activity of an atom and updates it's position on the activity heap, if the atom is on the activity heap.
pub fn bump_activity(
    atom: Atom,
    index_heap: &mut IndexHeap<Activity>,
    bumpy: &mut ConfigOption<Activity>,
) {
    index_heap.revalue(
        atom as usize,
        *index_heap.value_at(atom as usize) + bumpy.value,
    );
    index_heap.heapify_if_active(atom as usize);
}

/// Increase the activity bump applied to atoms by a factor.
pub fn exponent_activity(bumpy: &mut ConfigOption<Activity>, decay: &mut ConfigOption<Activity>) {
    let factor = 1.0 / (1.0 - decay.value);
    bumpy.value *= factor
}
