#[derive(Debug)]

pub struct FixedHeap<T: PartialOrd + Clone + Copy> {
    values: Vec<T>,
    map: Vec<Option<usize>>,
    heap: Vec<usize>,
    limit: usize,
}
use std::cmp::Ordering;

impl<T: PartialOrd + Clone + Copy + std::fmt::Debug + std::fmt::Display + std::ops::MulAssign>
    FixedHeap<T>
{
    pub fn new(size: usize, default_value: T) -> Self {
        FixedHeap {
            values: vec![default_value; size],
            map: vec![None; size],
            heap: vec![0; size],
            limit: 0,
        }
    }

    pub fn heap_index(&self, index: usize) -> usize {
        unsafe { *self.heap.get_unchecked(index) }
    }

    pub fn map_heap_index(&self, index: usize) -> Option<usize> {
        unsafe { *self.map.get_unchecked(*self.heap.get_unchecked(index)) }
    }

    pub fn heap_left(&self, index: usize) -> usize {
        (2 * index) + 1
    }

    pub fn heap_right(&self, index: usize) -> usize {
        (2 * index) + 2
    }

    pub fn heap_parent(&self, index: usize) -> usize {
        index.saturating_sub(1) / 2
    }

    pub fn parent(&self, index: usize) -> &T {
        self.values
            .get(self.heap_parent(index))
            .expect("missing parent")
    }

    pub fn position(&self, index: usize) -> Option<usize> {
        unsafe { *self.map.get_unchecked(index) }
    }

    #[allow(clippy::single_match)]
    pub fn heapify_down(&mut self, mut index: usize) {
        loop {
            let left_heap = self.heap_left(index);
            if left_heap >= self.limit {
                break;
            }
            let mut largest = index;
            let mut largest_value = self.values[self.heap[largest]];

            let left_value = self.values[self.heap[left_heap]];
            match left_value.partial_cmp(&largest_value) {
                Some(Ordering::Greater) => {
                    largest = left_heap;
                    largest_value = left_value;
                }
                _ => {}
            }
            let right_index = self.heap_right(index);
            if right_index < self.limit {
                let right_value = self.values[self.heap[right_index]];
                match right_value.partial_cmp(&largest_value) {
                    Some(Ordering::Greater) => {
                        largest = right_index;
                    }
                    _ => {}
                }
            }
            if largest != index {
                self.map[self.heap[largest]] = Some(index);
                self.map[self.heap[index]] = Some(largest);
                self.heap.swap(index, largest);
                index = largest;
            } else {
                break;
            }
        }
    }

    pub fn heapify_up(&mut self, mut index: usize) {
        loop {
            if index == 0 {
                break;
            }
            let parent_heap = self.heap_parent(index);

            let index_value = self.values[self.heap[index]];
            let parent_value = self.values[self.heap[parent_heap]];
            match parent_value.partial_cmp(&index_value) {
                Some(Ordering::Greater) => break,
                _ => {
                    self.map[self.heap[parent_heap]] = Some(index);
                    self.map[self.heap[index]] = Some(parent_heap);
                    self.heap.swap(index, parent_heap);
                    index = parent_heap;
                }
            }
        }
    }

    pub fn insert(&mut self, index: usize, value: T) {
        if unsafe { self.map.get_unchecked(index).is_none() } {
            self.values[index] = value;
            self.map[index] = Some(self.limit);
            self.heap[self.limit] = index;
            self.heapify_up(self.limit);
            self.limit += 1;
        }
    }

    pub fn activate(&mut self, index: usize) {
        if self.map[index].is_none() {
            self.map[index] = Some(self.limit);
            self.heap[self.limit] = index;
            self.heapify_up(self.limit);
            self.limit += 1;
        } else {
            let heap_index = self.map[index].unwrap();
            self.heapify_up(heap_index);
            self.heapify_down(heap_index);
        }
    }

    pub fn peek_max(&self) -> Option<usize> {
        match self.limit {
            0 => None,
            _ => {
                let index = *self.heap.first().unwrap();
                Some(index)
            }
        }
    }

    pub fn peek_max_value(&self) -> Option<T> {
        match self.limit {
            0 => None,
            _ => {
                let index = *self.heap.first().unwrap();
                Some(self.values[index])
            }
        }
    }

    pub fn pop_max(&mut self) -> Option<usize> {
        match self.limit {
            0 => None,
            1 => {
                let index = self.heap[0];
                self.map[self.heap[0]] = None;
                self.limit = 0;
                Some(index)
            }
            _ => {
                let max_index = self.heap[0];
                self.map[self.heap[0]] = None;
                self.limit -= 1;
                self.heap.swap(0, self.limit);
                self.map[self.heap[0]] = Some(0);
                self.heapify_down(0);
                Some(max_index)
            }
        }
    }

    pub fn bobble(&mut self) {
        for index in (0..self.limit / 2).rev() {
            self.heapify_down(index)
        }
    }

    pub fn value_at(&self, index: usize) -> T {
        self.values[index]
    }

    pub fn update_one(&mut self, index: usize, value: T) {
        self.values[index] = value;
        if let Some(heap_index) = self.map[index] {
            self.heapify_up(heap_index);
            self.heapify_down(heap_index);
        }
    }

    pub fn reduce_all_with(&mut self, factor: T) {
        for value in &mut self.values {
            *value *= factor;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
    struct TestStruct {
        value: usize,
        position: usize,
    }

    impl TestStruct {
        fn new(value: usize, position: usize) -> Self {
            Self { value, position }
        }
    }

    #[test]
    fn heap_simple() {
        let mut test_heap = FixedHeap::new(7, 0);
        test_heap.insert(6, 10);
        test_heap.insert(5, 20);
        test_heap.insert(4, 30);
        test_heap.insert(1, 60);
        test_heap.insert(0, 70);

        assert!(test_heap.pop_max().is_some_and(|max| max == 0));
        assert!(test_heap.pop_max().is_some_and(|max| max == 1));
        assert!(test_heap.pop_max().is_some_and(|max| max == 4));
        assert!(test_heap.pop_max().is_some_and(|max| max == 5));
        assert!(test_heap.pop_max().is_some_and(|max| max == 6));

        // assert_eq!(test_heap.position(0), Some(0));

        // assert_eq!(test_heap.peek_max().unwrap(), 70);
        // assert_eq!(test_heap.peek_max().unwrap().position, 0);

        // assert!(test_heap.pop_max().is_some_and(|max| max == 70));

        // assert_eq!(test_heap.values[test_heap.map[4].unwrap()].position, 4);

        // assert!(test_heap.pop_max().is_some_and(|max| max.value == 60));

        // assert_eq!(test_heap.values[test_heap.map[4].unwrap()].position, 4);

        // assert!(test_heap.pop_max().is_some_and(|max| max.value == 30));

        // assert_eq!(test_heap.map[4], None);

        // assert!(test_heap.pop_max().is_some_and(|max| max.value == 20));
        // assert!(test_heap.pop_max().is_some_and(|max| max.value == 10));
    }

    #[test]
    fn heap_update() {
        let mut test_heap = FixedHeap::new(7, 0);
        test_heap.insert(6, 10);
        test_heap.insert(5, 20);
        test_heap.insert(4, 30);
        test_heap.insert(1, 60);
        test_heap.insert(0, 70);

        test_heap.values[0] = 0;
        test_heap.values[1] = 1;
        test_heap.values[4] = 4;
        test_heap.values[5] = 5;
        test_heap.values[6] = 6;

        test_heap.bobble();

        assert!(test_heap.pop_max().is_some_and(|max| max == 6));
        assert!(test_heap.pop_max().is_some_and(|max| max == 5));
        assert!(test_heap.pop_max().is_some_and(|max| max == 4));
        assert!(test_heap.pop_max().is_some_and(|max| max == 1));
        assert!(test_heap.pop_max().is_some_and(|max| max == 0));

        test_heap.bobble();
    }
}
