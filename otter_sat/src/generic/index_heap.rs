/*!
A heap on some subset of elements with fixed indicies.

In other words, a heap backed by a vector with a companion vector which tracks the current location of the initial index of a heap element in the heap.

Further, the backing vector stays constant, allowing heap to act as a store of elements elements which may be moved onto the heap.

For example, [IndexHeap] is used as a store of [atoms](crate::structures::atom), as atoms are associated with an index and it is a useful heuristic to choose an atom without a value with the most activity when given a partial valuation with no queued consequences to expand on during a solve.

Further, to help maintain the heap callback functions [apply_to_value_at_value_index](IndexHeap::apply_to_value_at_value_index) and [apply_to_all](IndexHeap::apply_to_all) are provided.[^note]
[^note]: For example, [apply_to_value_at_value_index](IndexHeap::apply_to_value_at_value_index) allows increasing the activity of an atom and [apply_to_all](IndexHeap::apply_to_all) allows scaling the activity of all atoms.

```rust
# use otter_sat::generic::index_heap::IndexHeap;
let mut test_heap = IndexHeap::default();

        test_heap.add(600, 10);
        test_heap.add(0, 70);

        test_heap.activate(600);
        test_heap.activate(0);

 assert_eq!(test_heap.count(), 601);
 assert_eq!(test_heap.value_at(5), &i32::default());

 assert_eq!(test_heap.pop_max(), Some(0));
 assert_eq!(test_heap.pop_max(), Some(600));

 assert!(test_heap.pop_max().is_none());
*/

/// The index heap struct.
pub struct IndexHeap<V: PartialOrd + Default> {
    values: Vec<V>,
    position_in_heap: Vec<Option<usize>>,
    heap: Vec<usize>,
    limit: usize,
}
use std::cmp::Ordering;

impl<V: Default + PartialOrd + Default> Default for IndexHeap<V> {
    fn default() -> Self {
        IndexHeap {
            values: Vec::default(),
            position_in_heap: Vec::default(),
            heap: Vec::default(),
            limit: 0,
        }
    }
}

impl<V: PartialOrd + Default> IndexHeap<V> {
    /// Index `value` with `value_index`.
    /// Returns true if `value_index` was a fresh index, false otherwise.
    /// To *activate* `value_index` on the heap [activate]([IndexHeap::activate]) should be called after this method.
    ///
    /// Note, the method grows the structure to the size required for `value_index` to be a (transparent) index.
    pub fn add(&mut self, value_index: usize, value: V) -> bool {
        if self.heap.len() <= value_index {
            match (value_index - self.heap.len()) + 1 {
                1 => {
                    self.values.push(value); // Push the value.
                    self.position_in_heap.push(None); // The value_index is not on the heap, by default.
                    self.heap.push(usize::MAX); // As the index is beyond the limit of the heap, this may be any arbitrary value.
                }

                required => {
                    self.position_in_heap.append(&mut vec![None; required]);

                    let mut value_vec = Vec::with_capacity(required);
                    for _ in 0..required {
                        value_vec.push(V::default())
                    }

                    self.values.append(&mut value_vec);
                    self.heap.append(&mut vec![0; required]);
                    self.revalue(value_index, value);
                }
            }

            true
        } else {
            self.revalue(value_index, value);
            false
        }
    }

    /// Remove `value_index` from the heap, if present.
    /// Returns true if `value_index` was removed, false otherwise.
    pub fn remove(&mut self, value_index: usize) -> bool {
        unsafe {
            if let Some(heap_index) = self.heap_index(value_index) {
                if heap_index == self.limit - 1 {
                    self.limit -= 1;
                    self.reposition(value_index, None);
                } else if heap_index < self.limit {
                    self.limit -= 1;
                    self.reposition(self.value_index(self.limit), Some(heap_index));
                    self.heap.swap(heap_index, self.limit);
                    self.reposition(value_index, None);
                    self.heapify_down(heap_index);
                }
                true
            } else {
                false
            }
        }
    }

    /// Activate the value on the heap at `index`.
    pub fn activate(&mut self, index: usize) -> bool {
        unsafe {
            match self.heap_index(index) {
                None => {
                    self.reposition(index, Some(self.limit));
                    *self.heap.get_unchecked_mut(self.limit) = index;
                    self.heapify_up(self.limit);
                    self.limit += 1;
                    true
                }
                Some(heap_index) => {
                    self.heapify_up(heap_index);
                    self.heapify_down(heap_index);
                    false
                }
            }
        }
    }

    /// Heapify (ensure invariants of the heap are upheld) if `value_index` is active.
    pub fn heapify_if_active(&mut self, value_index: usize) {
        unsafe {
            if let Some(heap_index) = self.heap_index(value_index) {
                self.heapify_down(heap_index);
                self.heapify_up(heap_index);
            }
        }
    }

    /// Peak at the maximum index of the heap.
    pub fn peek_max(&self) -> Option<usize> {
        match self.limit {
            0 => None,
            _ => Some(unsafe { *self.heap.get_unchecked(0) }),
        }
    }

    /// Peak at the maximum value of the heap.
    pub fn peek_max_value(&self) -> Option<&V> {
        match self.peek_max() {
            None => None,
            Some(max_value_index) => Some(self.value_at(max_value_index)),
        }
    }

    /// Pop at the maximum index off the heap.
    pub fn pop_max(&mut self) -> Option<usize> {
        match self.limit {
            0 => None,
            _ => unsafe {
                let max_heap_index = self.value_index(0);
                self.remove(max_heap_index);
                Some(max_heap_index)
            },
        }
    }

    /// Heapify (ensure invariants of the heap are upheld) the heap.
    pub fn heapify(&mut self) {
        for heap_index in (0..self.limit / 2).rev() {
            unsafe { self.heapify_down(heap_index) }
        }
    }

    /// Return the value indexed by `value_index`.
    pub fn value_at(&self, value_index: usize) -> &V {
        unsafe { self.values.get_unchecked(value_index) }
    }

    /// Apply `f` to the value at `value_index`.
    pub fn apply_to_value_at_value_index(&mut self, value_index: usize, f: impl Fn(&V) -> V) {
        unsafe {
            *self.values.get_unchecked_mut(value_index) = f(self.values.get_unchecked(value_index))
        }
    }

    /// Apply `f` to all (indexed) values.
    pub fn apply_to_all(&mut self, f: impl Fn(&V) -> V) {
        for value in self.values.iter_mut() {
            *value = f(value)
        }
    }

    /// Set the value of `value_index` to `value.
    pub fn revalue(&mut self, value_index: usize, value: V) {
        unsafe { *self.values.get_unchecked_mut(value_index) = value }
    }

    /// A count of values indexed by the structure.
    pub fn count(&self) -> usize {
        self.values.len()
    }

    /// True if the heap is empty, false otherwise.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl<V: PartialOrd + Default> IndexHeap<V> {
    /// The index of some value stored at `heap_index` on the heap.
    ///
    /// # Safety
    /// Assumes `heap_index` is some location on the heap.
    unsafe fn value_index(&self, heap_index: usize) -> usize {
        *unsafe { self.heap.get_unchecked(heap_index) }
    }

    /// Where `value_index` is stored on the heap, if present.
    unsafe fn heap_index(&self, value_index: usize) -> Option<usize> {
        *unsafe { self.position_in_heap.get_unchecked(value_index) }
    }

    /// Updates the position in the heap of `value_index` to `heap_index`.
    unsafe fn reposition(&mut self, value_index: usize, heap_index: Option<usize>) {
        *unsafe { self.position_in_heap.get_unchecked_mut(value_index) } = heap_index;
    }

    /// The (heap) index of the left child of `heap_index`.
    fn heap_left(&self, heap_index: usize) -> usize {
        (2 * heap_index) + 1
    }

    /// The (heap) index of the right child of `heap_index`.
    fn heap_right(&self, heap_index: usize) -> usize {
        (2 * heap_index) + 2
    }

    /// The (heap) index of the parent of `heap_index`.
    fn heap_parent(&self, heap_index: usize) -> usize {
        heap_index.saturating_sub(1) / 2
    }

    /// Shuffles the index down into the heap, if required.
    ///
    /// The method is a typical implementation of heapify down.
    /// Though, with added steps to perform comparisons on values stored.
    ///
    /// The goal is to ensure a max heap, and so for any trio of an index, the left child of the index, and the right child, two pai rwaise comparisons are made.
    /// 1. Between the index and the left index, as if the left index is larger the index should (at least) be pushed to the left.
    /// 2. Between the index and the right index, as if the right index is larger the index should be pushed to the right.
    ///
    /// The implemention maintains a note of the update index to use, and revises this to the left and right as needed.
    /// Alterantive implementation may first identify the largest child and then determine the updated index.
    ///
    /// Only after the required update has been identified is an update made.
    unsafe fn heapify_down(&mut self, mut heap_index: usize) {
        let mut left_index;
        let mut left_value;

        let mut right_index;
        // The right value is only accessed once in a loop.

        let mut update_index;
        let mut update_value;

        // Temporary swap storage.
        let mut a;
        let mut b;

        loop {
            left_index = self.heap_left(heap_index);
            if left_index >= self.limit {
                break;
            }
            left_value = unsafe { self.values.get_unchecked(self.value_index(left_index)) };

            update_index = heap_index;
            update_value = unsafe { self.values.get_unchecked(self.value_index(update_index)) };

            if left_value > update_value {
                update_index = left_index;
                update_value = left_value;
            }

            right_index = self.heap_right(heap_index);

            if right_index < self.limit
                && unsafe { self.values.get_unchecked(self.value_index(right_index)) }
                    > update_value
            {
                update_index = right_index;
            }

            if update_index != heap_index {
                a = unsafe { self.value_index(heap_index) };
                b = unsafe { self.value_index(update_index) };

                self.position_in_heap.swap(a, b);
                self.heap.swap(heap_index, update_index);

                heap_index = update_index;
            } else {
                break;
            }
        }
    }

    /// Shuffles the index up from the heap, if required.
    ///
    /// Swaps the index with it's parent in the heap, if the parent is smaller.
    unsafe fn heapify_up(&mut self, mut heap_index: usize) {
        let mut parent_heap;

        let mut index_value;
        let mut parent_value;

        // Temporary swap storage.
        let mut a;
        let mut b;

        'up_loop: loop {
            if heap_index == 0 {
                break 'up_loop;
            }
            parent_heap = self.heap_parent(heap_index);

            index_value = unsafe { self.values.get_unchecked(self.value_index(heap_index)) };
            parent_value = unsafe { self.values.get_unchecked(self.value_index(parent_heap)) };

            match parent_value.partial_cmp(index_value) {
                Some(Ordering::Greater) => break 'up_loop,
                _ => {
                    a = unsafe { self.value_index(heap_index) };
                    b = unsafe { self.value_index(parent_heap) };

                    self.position_in_heap.swap(a, b);
                    self.heap.swap(heap_index, parent_heap);

                    heap_index = parent_heap;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heap_simple() {
        let mut test_heap = IndexHeap::default();
        test_heap.add(6, 10);
        test_heap.add(5, 20);
        test_heap.add(4, 30);
        test_heap.add(1, 60);
        test_heap.add(0, 70);
        test_heap.activate(6);
        test_heap.activate(5);
        test_heap.activate(4);
        test_heap.activate(1);
        test_heap.activate(0);

        assert_eq!(test_heap.pop_max(), Some(0));
        assert_eq!(test_heap.pop_max(), Some(1));
        assert_eq!(test_heap.pop_max(), Some(4));
        assert_eq!(test_heap.pop_max(), Some(5));
        assert_eq!(test_heap.pop_max(), Some(6));
    }

    #[test]
    fn heap_update() {
        let mut test_heap = IndexHeap::default();
        test_heap.add(6, 10);
        test_heap.add(4, 30);
        test_heap.add(1, 60);
        test_heap.add(0, 70);
        test_heap.activate(6);
        test_heap.activate(4);
        test_heap.activate(1);
        test_heap.activate(0);

        test_heap.values[0] = 0;
        test_heap.values[1] = 1;
        test_heap.values[4] = 4;
        test_heap.values[6] = 6;

        test_heap.heapify();

        assert_eq!(test_heap.pop_max(), Some(6));
        assert_eq!(test_heap.pop_max(), Some(4));
        assert_eq!(test_heap.pop_max(), Some(1));
        assert_eq!(test_heap.pop_max(), Some(0));
        assert!(test_heap.pop_max().is_none());

        test_heap.heapify();
    }

    #[test]
    fn heap_sparse() {
        let mut test_heap = IndexHeap::default();
        test_heap.add(600, 10);
        test_heap.add(0, 70);
        test_heap.activate(600);
        test_heap.activate(0);

        assert_eq!(test_heap.values.len(), 601);
        assert_eq!(test_heap.values[5], i32::default());
        assert_eq!(test_heap.pop_max(), Some(0));
        assert_eq!(test_heap.pop_max(), Some(600));
        assert!(test_heap.pop_max().is_none());
    }

    #[test]
    fn heap_remove() {
        let mut test_heap = IndexHeap::default();
        test_heap.add(6, 6);
        test_heap.add(5, 5);
        test_heap.add(4, 4);
        test_heap.add(1, 1);
        test_heap.add(0, 0);
        test_heap.activate(6);
        test_heap.activate(5);
        test_heap.activate(4);
        test_heap.activate(1);
        test_heap.activate(0);

        assert!(test_heap.remove(4));
        assert!(!test_heap.remove(4));
        assert!(test_heap.remove(6));
        assert!(!test_heap.add(4, 10));
        assert!(!test_heap.add(4, 1));
        test_heap.activate(4);

        assert_eq!(test_heap.pop_max(), Some(5));
        assert_eq!(test_heap.pop_max(), Some(1));
        assert_eq!(test_heap.pop_max(), Some(4));
        assert_eq!(test_heap.pop_max(), Some(0));
    }
}
