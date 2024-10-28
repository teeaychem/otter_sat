#[derive(Debug)]
pub struct FixedHeap<T: Ord> {
    structure: Vec<T>,
    positions: Vec<Option<usize>>,
}
use crate::generic::fixed_index::FixedIndex;
use std::cmp::Ordering;

impl<T: Ord + FixedIndex> FixedHeap<T> {
    pub fn new(size: usize) -> Self {
        FixedHeap {
            structure: Vec::with_capacity(size),
            positions: vec![None; size],
        }
    }

    pub fn left_index(&self, index: usize) -> usize {
        (2 * index) + 1
    }

    pub fn right_index(&self, index: usize) -> usize {
        (2 * index) + 2
    }

    pub fn parent_index(&self, index: usize) -> usize {
        index.saturating_sub(1) / 2
    }

    pub fn left(&self, index: usize) -> Option<&T> {
        self.structure.get(self.left_index(index))
    }

    pub fn right(&self, index: usize) -> Option<&T> {
        self.structure.get(self.right_index(index))
    }

    pub fn parent(&self, index: usize) -> &T {
        self.structure
            .get(self.parent_index(index))
            .expect("missing parent")
    }

    pub fn position(&self, index: usize) -> Option<usize> {
        unsafe { *self.positions.get_unchecked(index) }
    }

    pub fn heapify_down(&mut self, mut index: usize) {
        loop {
            let index_item_fixed_index = self
                .structure
                .get(index)
                .expect("missing index item")
                .index();
            let mut largest = index;
            let mut largest_item = self.structure.get(largest).expect("missing index item");
            let mut largest_item_fixed_index = largest_item.index();
            if let Some(entry) = self.left(index) {
                match largest_item.cmp(entry) {
                    Ordering::Less | Ordering::Equal => {
                        largest = self.left_index(index);
                        largest_item = self.structure.get(largest).expect("missing index item");
                        largest_item_fixed_index = largest_item.index();
                    }
                    Ordering::Greater => {}
                }
            } else {
                break;
            }
            if let Some(entry) = self.right(index) {
                match largest_item.cmp(entry) {
                    Ordering::Less | Ordering::Equal => {
                        largest = self.right_index(index);
                        largest_item = self.structure.get(largest).expect("missing index item");
                        largest_item_fixed_index = largest_item.index();
                    }
                    Ordering::Greater => {}
                }
            }
            if largest != index {
                self.structure.swap(largest, index);
                unsafe {
                    *self.positions.get_unchecked_mut(index_item_fixed_index) = Some(largest);
                    *self.positions.get_unchecked_mut(largest_item_fixed_index) = Some(index);
                }

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
            let index_item = self.structure.get(index).expect("missing index");
            let index_item_fixed_index = index_item.index();
            let parent_item = self.parent(index);
            let parent_item_fixed_index = parent_item.index();
            match index_item.cmp(parent_item) {
                Ordering::Less => break,
                _ => {
                    let parent_index = self.parent_index(index);
                    self.structure.swap(index, parent_index);
                    unsafe {
                        *self.positions.get_unchecked_mut(index_item_fixed_index) =
                            Some(parent_index);
                        *self.positions.get_unchecked_mut(parent_item_fixed_index) = Some(index);
                    }
                    index = parent_index;
                }
            }
        }
    }

    pub fn insert(&mut self, item: T) {
        let item_index = item.index();
        if unsafe { self.positions.get_unchecked(item_index).is_none() } {
            let end_index = self.structure.len();
            self.structure.push(item);
            unsafe {
                *self.positions.get_unchecked_mut(item_index) = Some(end_index);
            }

            self.heapify_up(end_index);
        }
    }

    pub fn peek_max(&self) -> Option<&T> {
        self.structure.first()
    }

    pub fn pop_max(&mut self) -> Option<T> {
        match self.structure.len() {
            0 => None,
            1 => {
                let max_item = self.structure.swap_remove(0);
                unsafe { *self.positions.get_unchecked_mut(max_item.index()) = None };
                Some(max_item)
            }
            _ => {
                let max_item = self.structure.swap_remove(0);
                unsafe {
                    let new_first_fixed_index = self.structure.get_unchecked(0).index();
                    *self.positions.get_unchecked_mut(new_first_fixed_index) = Some(0);
                    *self.positions.get_unchecked_mut(max_item.index()) = None;
                }
                self.heapify_down(0);
                Some(max_item)
            }
        }
    }

    pub fn bobble(&mut self) {
        for index in (0..self.structure.len()).rev() {
            self.heapify_up(index)
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

    impl FixedIndex for TestStruct {
        fn index(&self) -> usize {
            self.position
        }
    }

    impl TestStruct {
        fn new(value: usize, position: usize) -> Self {
            Self { value, position }
        }
    }

    #[test]
    fn heap_simple() {
        let mut test_heap = FixedHeap::new(7);
        test_heap.insert(TestStruct::new(10, 6));
        test_heap.insert(TestStruct::new(20, 5));
        test_heap.insert(TestStruct::new(30, 4));
        test_heap.insert(TestStruct::new(60, 1));
        test_heap.insert(TestStruct::new(70, 0));

        assert_eq!(test_heap.position(0), Some(0));

        assert_eq!(test_heap.peek_max().unwrap().value, 70);
        assert_eq!(test_heap.peek_max().unwrap().position, 0);

        assert!(test_heap.pop_max().is_some_and(|max| max.value == 70));

        assert_eq!(
            test_heap.structure[test_heap.positions[4].unwrap()].position,
            4
        );

        assert!(test_heap.pop_max().is_some_and(|max| max.value == 60));

        assert_eq!(
            test_heap.structure[test_heap.positions[4].unwrap()].position,
            4
        );

        assert!(test_heap.pop_max().is_some_and(|max| max.value == 30));

        assert_eq!(test_heap.positions[4], None);

        assert!(test_heap.pop_max().is_some_and(|max| max.value == 20));
        assert!(test_heap.pop_max().is_some_and(|max| max.value == 10));
    }

    #[test]
    fn heap_update() {
        let mut test_heap = FixedHeap::new(7);
        test_heap.insert(TestStruct::new(10, 6));
        test_heap.insert(TestStruct::new(20, 5));
        test_heap.insert(TestStruct::new(30, 4));
        test_heap.insert(TestStruct::new(60, 1));
        test_heap.insert(TestStruct::new(70, 0));

        test_heap.structure[test_heap.positions[0].unwrap()].value = 0;
        test_heap.structure[test_heap.positions[1].unwrap()].value = 1;
        test_heap.structure[test_heap.positions[4].unwrap()].value = 4;
        test_heap.structure[test_heap.positions[5].unwrap()].value = 5;
        test_heap.structure[test_heap.positions[6].unwrap()].value = 6;

        test_heap.bobble();
    }
}
