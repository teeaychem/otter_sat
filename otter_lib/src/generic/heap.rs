#[derive(Debug)]

pub struct IndexHeap<V: PartialOrd + Default> {
    values: Vec<V>,
    position_in_heap: Vec<Option<usize>>,
    heap: Vec<usize>,
    pub limit: usize,
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
    pub fn add(&mut self, index: usize, value: V) -> bool {
        if self.heap.is_empty() || index > self.heap.len() - 1 {
            let required = (index - self.heap.len()) + 1;
            self.position_in_heap.append(&mut vec![None; required]);

            let mut value_vec = Vec::with_capacity(required);
            for _ in 0..required {
                value_vec.push(V::default())
            }

            self.values.append(&mut value_vec);
            self.heap.append(&mut vec![0; required]);
            self.revalue(index, value);
            true
        } else {
            self.revalue(index, value);
            false
        }
    }

    pub fn remove(&mut self, index: usize) -> bool {
        unsafe {
            if let Some(heap_position) = self.position(index) {
                if heap_position == self.limit - 1 {
                    self.limit -= 1;
                    self.reposition(index, None);
                } else if heap_position < self.limit {
                    self.limit -= 1;
                    self.reposition(self.heap_index(self.limit), Some(heap_position));
                    self.heap.swap(heap_position, self.limit);
                    self.reposition(index, None);
                    self.heapify_down(heap_position);
                }
                true
            } else {
                false
            }
        }
    }

    pub fn activate(&mut self, index: usize) -> bool {
        unsafe {
            match self.position(index) {
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

    pub fn heapify_if_active(&mut self, index: usize) {
        unsafe {
            if let Some(heap_index) = self.position(index) {
                self.heapify_down(heap_index);
                self.heapify_up(heap_index);
            }
        }
    }

    pub fn peek_max(&self) -> Option<usize> {
        match self.limit {
            0 => None,
            _ => Some(unsafe { *self.heap.get_unchecked(0) }),
        }
    }

    pub fn peek_max_value(&self) -> Option<&V> {
        match self.limit {
            0 => None,
            _ => Some(self.value_at(self.peek_max().unwrap())),
        }
    }

    pub fn pop_max(&mut self) -> Option<usize> {
        match self.limit {
            0 => None,
            _ => unsafe {
                let max_index = self.heap_index(0);
                self.remove(max_index);
                Some(max_index)
            },
        }
    }

    pub fn reheap(&mut self) {
        for index in (0..self.limit / 2).rev() {
            unsafe { self.heapify_down(index) }
        }
    }

    pub fn value_at(&self, index: usize) -> &V {
        unsafe { self.values.get_unchecked(index) }
    }

    pub fn update_one(&mut self, index: usize, value: V) {
        self.revalue(index, value);
        self.heapify_if_active(index);
    }

    pub fn apply_to_index(&mut self, index: usize, f: impl Fn(&V) -> V) {
        unsafe { *self.values.get_unchecked_mut(index) = f(self.values.get_unchecked(index)) }
    }

    pub fn apply_to_all(&mut self, f: impl Fn(&V) -> V) {
        for value in &mut self.values {
            *value = f(value)
        }
    }

    pub fn revalue(&mut self, index: usize, value: V) {
        unsafe { *self.values.get_unchecked_mut(index) = value }
    }
}

impl<V: PartialOrd + Default> IndexHeap<V> {
    unsafe fn heap_index(&self, index: usize) -> usize {
        *self.heap.get_unchecked(index)
    }

    unsafe fn position(&self, index: usize) -> Option<usize> {
        *self.position_in_heap.get_unchecked(index)
    }

    unsafe fn reposition(&mut self, from: usize, to: Option<usize>) {
        *self.position_in_heap.get_unchecked_mut(from) = to;
    }

    fn heap_left(&self, index: usize) -> usize {
        (2 * index) + 1
    }

    fn heap_right(&self, index: usize) -> usize {
        (2 * index) + 2
    }

    fn heap_parent(&self, index: usize) -> usize {
        index.saturating_sub(1) / 2
    }

    #[allow(clippy::single_match)]
    unsafe fn heapify_down(&mut self, mut index: usize) {
        loop {
            let left_index = self.heap_left(index);
            if left_index >= self.limit {
                break;
            }
            let mut largest = index;
            let mut largest_value = self.values.get_unchecked(self.heap_index(largest));

            let left_value = self.values.get_unchecked(self.heap_index(left_index));
            match left_value.partial_cmp(largest_value) {
                Some(Ordering::Greater) => {
                    largest = left_index;
                    largest_value = left_value;
                }
                _ => {}
            }
            let right_index = self.heap_right(index);
            if right_index < self.limit {
                let right_value = self.values.get_unchecked(self.heap_index(right_index));
                match right_value.partial_cmp(largest_value) {
                    Some(Ordering::Greater) => {
                        largest = right_index;
                    }
                    _ => {}
                }
            }
            if largest != index {
                self.reposition(self.heap_index(largest), Some(index));
                self.reposition(self.heap_index(index), Some(largest));
                self.heap.swap(index, largest);
                index = largest;
            } else {
                break;
            }
        }
    }

    unsafe fn heapify_up(&mut self, mut index: usize) {
        loop {
            if index == 0 {
                break;
            }
            let parent_heap = self.heap_parent(index);

            let index_value = self.values.get_unchecked(self.heap_index(index));
            let parent_value = self.values.get_unchecked(self.heap_index(parent_heap));
            match parent_value.partial_cmp(index_value) {
                Some(Ordering::Greater) => break,
                _ => {
                    let parent_heap_index = self.heap_index(parent_heap);

                    self.reposition(parent_heap_index, Some(index));
                    let heap_index = self.heap_index(index);
                    self.reposition(heap_index, Some(parent_heap));
                    self.heap.swap(index, parent_heap);
                    index = parent_heap;
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

        assert_eq!(test_heap.pop_max().unwrap(), 0);
        assert_eq!(test_heap.pop_max().unwrap(), 1);
        assert_eq!(test_heap.pop_max().unwrap(), 4);
        assert_eq!(test_heap.pop_max().unwrap(), 5);
        assert_eq!(test_heap.pop_max().unwrap(), 6);
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

        test_heap.reheap();

        assert_eq!(test_heap.pop_max().unwrap(), 6);
        assert_eq!(test_heap.pop_max().unwrap(), 4);
        assert_eq!(test_heap.pop_max().unwrap(), 1);
        assert_eq!(test_heap.pop_max().unwrap(), 0);
        assert!(test_heap.pop_max().is_none());

        test_heap.reheap();
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
        assert_eq!(test_heap.pop_max().unwrap(), 0);
        assert_eq!(test_heap.pop_max().unwrap(), 600);
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

        assert_eq!(test_heap.pop_max().unwrap(), 5);
        assert_eq!(test_heap.pop_max().unwrap(), 1);
        assert_eq!(test_heap.pop_max().unwrap(), 4);
        assert_eq!(test_heap.pop_max().unwrap(), 0);
    }
}