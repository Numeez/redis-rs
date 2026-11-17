
pub struct MinHeap<T> {
    data: Vec<T>,
}

impl<T: Ord> MinHeap<T> {
    pub fn new() -> Self {
        Self { data: vec![] }
    }

    pub fn insert(&mut self, value: T) {
        self.data.push(value);
        let length = self.data.len();
        if length > 1 {
            let mut current_idx = length - 1;
            let mut parent_idx = Self::get_parent_index(current_idx);
            while (self.data[parent_idx] > self.data[current_idx]) && current_idx > 0 {
                self.data.swap(parent_idx, current_idx);
                if parent_idx == 0 {
                    break;
                }
                current_idx = parent_idx;
                parent_idx = Self::get_parent_index(current_idx);
            }
        }
    }
    pub fn delete(&mut self) -> Option<T> {
        if self.data.is_empty() {
            return None;
        }

        let last = self.data.pop().unwrap();
        if self.data.is_empty() {
            return Some(last);
        }

        let min = std::mem::replace(&mut self.data[0], last);

        let mut current_idx = 0;
        loop {
            let left_idx = Self::get_left_child_index(current_idx);
            let right_idx = Self::get_right_child_index(current_idx);
            let length = self.data.len();

            if left_idx >= length {
                break;
            }

            let mut result_idx = left_idx;
            if right_idx < length && self.data[right_idx] < self.data[left_idx] {
                result_idx = right_idx;
            }

            if self.data[current_idx] > self.data[result_idx] {
                self.data.swap(current_idx, result_idx);
                current_idx = result_idx;
            } else {
                break;
            }
        }

        Some(min)
    }
    pub fn peek(&self) -> Option<&T> {
        if self.data.is_empty() {
            return None;
        }
        Some(&self.data[0])
    }
    fn get_parent_index(current_index: usize) -> usize {
        let idx = (current_index - 1) / 2;
        idx
    }
    fn get_left_child_index(current_index: usize) -> usize {
        let idx = 2 * current_index + 1;
        idx
    }
    fn get_right_child_index(current_index: usize) -> usize {
        let idx = 2 * current_index + 2;
        idx
    }
}
fn check_heap(heap: &MinHeap<usize>) {
    for i in 1..heap.data.len() {
        let parent = (i - 1) / 2;
        assert!(
            heap.data[parent] <= heap.data[i],
            "heap broken at index {}",
            i
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_heap_insert() {
        let mut heap = MinHeap::new();
        heap.insert(1);
        assert_eq!(vec![1], heap.data);
        heap.insert(3);
        heap.insert(5);
        assert_eq!(vec![1, 3, 5], heap.data);
        heap.insert(0);
        assert_eq!(vec![0, 1, 5, 3], heap.data);
    }
    #[test]
    fn test_min_heap_insert_2() {
        let mut heap = MinHeap::new();
        heap.insert(5);
        heap.insert(3);
        heap.insert(8);
        heap.insert(1);
        heap.insert(4);
        assert_eq!(vec![1, 3, 8, 5, 4], heap.data);
    }
    #[test]
    fn test_min_heap_insert_3() {
        let mut heap = MinHeap::new();
        heap.insert(5);
        heap.insert(4);
        heap.insert(3);
        heap.insert(2);
        heap.insert(1);
        assert_eq!(vec![1, 2, 4, 5, 3], heap.data);
    }
    #[test]
    fn test_min_heap_insert_4() {
        let mut heap = MinHeap::new();
        heap.insert(2);
        heap.insert(2);
        heap.insert(2);

        assert_eq!(vec![2, 2, 2], heap.data);
    }
    #[test]
    fn test_min_heap_insert_5() {
        let mut heap = MinHeap::new();
        heap.insert(1);
        heap.insert(2);
        heap.insert(3);
        heap.insert(4);
        assert_eq!(vec![1, 2, 3, 4], heap.data);
    }
    #[test]
    fn delete_from_empty() {
        let mut heap: MinHeap<usize> = MinHeap::new();
        assert_eq!(heap.delete(), None);
    }

    #[test]
    fn delete_single() {
        let mut heap = MinHeap::new();
        heap.insert(7);
        assert_eq!(heap.delete(), Some(7));
        assert_eq!(heap.delete(), None);
    }

    #[test]
    fn delete_returns_sorted() {
        let mut heap = MinHeap::new();
        for v in [5, 3, 8, 1, 4, 9, 2, 7, 6, 0] {
            heap.insert(v);
            check_heap(&heap);
        }
        let mut out = Vec::new();
        while let Some(v) = heap.delete() {
            check_heap(&heap);
            out.push(v);
        }
        assert_eq!(out, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn delete_with_duplicates() {
        let mut heap = MinHeap::new();
        for v in [3, 1, 3, 1, 2] {
            heap.insert(v);
        }
        let mut out = Vec::new();
        while let Some(v) = heap.delete() {
            out.push(v);
        }
        assert_eq!(out, vec![1, 1, 2, 3, 3]);
    }

    #[test]
    fn insert_after_delete() {
        let mut heap = MinHeap::new();
        for v in [4, 2, 6] {
            heap.insert(v);
        }
        assert_eq!(heap.delete(), Some(2));
        heap.insert(1);
        heap.insert(5);
        check_heap(&heap);
        assert_eq!(heap.delete(), Some(1));
        assert_eq!(heap.delete(), Some(4));
    }
}
