use std::process::id;

struct Min_Heap {
    data: Vec<usize>,
}

impl Min_Heap {
    pub fn new() -> Self {
        Self { data: vec![] }
    }

    pub fn insert(&mut self, value: usize) {
        self.data.push(value);
        let length = self.data.len();
        if length > 1 {
            let mut current_idx = length - 1;
            let mut parent_idx = Self::get_parent_index(current_idx);
            while (self.data[parent_idx] > self.data[current_idx]) && current_idx > 0 {
                let temp = self.data[parent_idx];
                self.data[parent_idx] = self.data[current_idx];
                self.data[current_idx] = temp;
                if parent_idx == 0 {
                    break;
                }
                current_idx = parent_idx;
                parent_idx = Self::get_parent_index(current_idx);
            }
        }
    }
    pub fn delete(&mut self) -> Option<usize> {
        if self.data.is_empty() {
            return None;
        }

        let min = self.data[0];

        let last = self.data.pop().unwrap();
        if self.data.is_empty() {
            return Some(min);
        }
        self.data[0] = last;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_min_heap_insert() {
        let mut heap = Min_Heap::new();
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
        let mut heap = Min_Heap::new();
        heap.insert(5);
        heap.insert(3);
        heap.insert(8);
        heap.insert(1);
        heap.insert(4);
        assert_eq!(vec![1, 3, 8, 5, 4], heap.data);
    }
    #[test]
    fn test_min_heap_insert_3() {
        let mut heap = Min_Heap::new();
        heap.insert(5);
        heap.insert(4);
        heap.insert(3);
        heap.insert(2);
        heap.insert(1);
        assert_eq!(vec![1, 2, 4, 5, 3], heap.data);
    }
    #[test]
    fn test_min_heap_insert_4() {
        let mut heap = Min_Heap::new();
        heap.insert(2);
        heap.insert(2);
        heap.insert(2);

        assert_eq!(vec![2, 2, 2], heap.data);
    }
    #[test]
    fn test_min_heap_insert_5() {
        let mut heap = Min_Heap::new();
        heap.insert(1);
        heap.insert(2);
        heap.insert(3);
        heap.insert(4);
        assert_eq!(vec![1, 2, 3, 4], heap.data);
    }
}
