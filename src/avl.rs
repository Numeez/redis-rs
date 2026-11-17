use crate::key::Key;
use std::cmp::Ordering;

#[derive(Debug)]
pub struct Node {
    key: Key,
    height: u32,
    count: u32,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

#[derive(Debug)]
pub struct Tree {
    root: Option<Box<Node>>,
}
impl Tree {
    pub fn new() -> Self {
        Self { root: None }
    }
    pub fn insert(&mut self, key: Key) {
        self.root = insert(self.root.take(), key)
    }
    pub fn delete(&mut self, key: Key) {
        self.root = delete(self.root.take(), key)
    }
    pub fn range(&self, start: &Key, offset: u32, limit: u32) -> Vec<&Key> {
        let begin = rank(&self.root, start) + offset;
        let mut out = Vec::new();
        for pos in begin..begin + limit {
            match select(&self.root, pos) {
                Some(key) => out.push(key),
                None => break,
            }
        }
        out
    }
}

impl Node {
    pub fn new(key: Key) -> Self {
        Self {
            key,
            height: 1,
            count: 1,
            left: None,
            right: None,
        }
    }
    fn update(&mut self) {
        self.height = 1 + height(&self.left).max(height(&self.right));
        self.count = 1 + count(&self.left) + count(&self.right);
    }
}

pub fn height(node: &Option<Box<Node>>) -> u32 {
    node.as_ref().map_or(0, |n| n.height)
}
pub fn count(node: &Option<Box<Node>>) -> u32 {
    node.as_ref().map_or(0, |n| n.count)
}

fn insert(node: Option<Box<Node>>, key: Key) -> Option<Box<Node>> {
    match node {
        None => Some(Box::new(Node::new(key))),
        Some(mut val) => {
            match key.cmp(&val.key) {
                Ordering::Less => val.left = insert(val.left.take(), key),
                Ordering::Greater => val.right = insert(val.right.take(), key),
                Ordering::Equal => {}
            }

            balance(val)
        }
    }
}

fn delete(node: Option<Box<Node>>, key: Key) -> Option<Box<Node>> {
    match node {
        None => None,
        Some(mut val) => match key.cmp(&val.key) {
            Ordering::Less => {
                val.left = delete(val.left.take(), key);
                balance(val)
            }
            Ordering::Greater => {
                val.right = delete(val.right.take(), key);
                balance(val)
            }
            Ordering::Equal => match (val.left.take(), val.right.take()) {
                (None, None) => None,
                (Some(l), None) => Some(l),
                (None, Some(r)) => Some(r),
                (Some(l), Some(r)) => {
                    let (rest, mut min) = take_min(r);
                    min.left = Some(l);
                    min.right = rest;
                    balance(min)
                }
            },
        },
    }
}

fn balance(mut val: Box<Node>) -> Option<Box<Node>> {
    val.update();
    let lh = height(&val.left);
    let rh = height(&val.right);

    if lh > rh + 1 {
        let left = val.left.as_ref().unwrap();
        let leans_right = height(&left.right) > height(&left.left);
        if leans_right {
            val.left = rotate_left(val.left.take());
        }
        return rotate_right(Some(val));
    }

    if rh > lh + 1 {
        let right = val.right.as_ref().unwrap();
        let leans_left = height(&right.left) > height(&right.right);
        if leans_left {
            val.right = rotate_right(val.right.take());
        }
        return rotate_left(Some(val));
    }

    Some(val)
}

fn take_min(mut val: Box<Node>) -> (Option<Box<Node>>, Box<Node>) {
    match val.left.take() {
        None => (val.right.take(), val),
        Some(left) => {
            let (rest, min) = take_min(left);
            val.left = rest;
            (balance(val), min)
        }
    }
}

fn _inorder(node: &Option<Box<Node>>, out: &mut Vec<Key>) {
    match node {
        None => {}
        Some(n) => {
            _inorder(&n.left, out);
            out.push(n.key.clone());
            _inorder(&n.right, out);
        }
    }
}
fn rotate_left(node: Option<Box<Node>>) -> Option<Box<Node>> {
    match node {
        None => None,
        Some(mut val) => {
            let mut right_child = val.right.take().expect("rotate_left: no right child");
            let x = right_child.left.take();
            val.right = x;
            val.update();
            right_child.left = Some(val);
            right_child.update();
            Some(right_child)
        }
    }
}

fn rotate_right(node: Option<Box<Node>>) -> Option<Box<Node>> {
    match node {
        None => None,
        Some(mut val) => {
            let mut left_child = val.left.take().expect("rotate_right: no left child");
            let x = left_child.right.take();
            val.left = x;
            val.update();
            left_child.right = Some(val);
            left_child.update();
            Some(left_child)
        }
    }
}

fn check(node: &Option<Box<Node>>) -> (u32, u32) {
    match node {
        None => (0, 0),
        Some(val) => {
            let (lh, lc) = check(&val.left);
            let (rh, rc) = check(&val.right);
            let h = 1 + lh.max(rh);
            let c = 1 + lc + rc;

            assert_eq!(val.height, h, "wrong height at key {}", val.key);
            assert_eq!(val.count, c, "wrong count at key {}", val.key);
            assert!(lh.abs_diff(rh) <= 1, "unbalanced at key {}", val.key);

            (h, c)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn k(n: u32) -> Key {
        Key::new(n as f64, &n.to_string())
    }

    fn verify(tree: &Tree, reference: &BTreeSet<u32>) {
        check(&tree.root);
        let mut out: Vec<Key> = Vec::new();
        _inorder(&tree.root, &mut out);
        let expected: Vec<Key> = reference.iter().map(|&n| k(n)).collect();
        assert_eq!(out, expected);
    }

    #[test]
    fn empty_tree() {
        let mut tree = Tree::new();
        check(&tree.root);
        tree.delete(k(1));
        assert!(tree.root.is_none());
    }

    #[test]
    fn sorted_inserts_stay_balanced() {
        let mut tree = Tree::new();
        for n in 1..=5 {
            tree.insert(k(n));
            check(&tree.root);
        }
        assert_eq!(height(&tree.root), 3);
    }

    #[test]
    fn duplicate_insert_is_ignored() {
        let mut tree = Tree::new();
        tree.insert(k(1));
        tree.insert(k(1));
        check(&tree.root);
        assert_eq!(count(&tree.root), 1);
    }

    #[test]
    fn delete_missing_key() {
        let mut tree = Tree::new();
        for n in 1..=3 {
            tree.insert(k(n));
        }
        tree.delete(k(99));
        check(&tree.root);
        assert_eq!(count(&tree.root), 3);
    }

    #[test]
    fn delete_two_children() {
        let mut tree = Tree::new();
        for n in 1..=7 {
            tree.insert(k(n));
        }
        tree.delete(k(4));
        check(&tree.root);
        assert_eq!(count(&tree.root), 6);
    }

    #[test]
    fn same_score_sorted_by_name() {
        let mut tree = Tree::new();
        tree.insert(Key::new(1.0, "carol"));
        tree.insert(Key::new(1.0, "alice"));
        tree.insert(Key::new(1.0, "bob"));
        tree.insert(Key::new(0.5, "zed"));
        check(&tree.root);

        let mut out = Vec::new();
        _inorder(&tree.root, &mut out);
        let expected = vec![
            Key::new(0.5, "zed"),
            Key::new(1.0, "alice"),
            Key::new(1.0, "bob"),
            Key::new(1.0, "carol"),
        ];
        assert_eq!(out, expected);

        tree.delete(Key::new(2.0, "bob"));
        assert_eq!(count(&tree.root), 4);
        tree.delete(Key::new(1.0, "bob"));
        assert_eq!(count(&tree.root), 3);
    }

    #[test]
    fn insert_at_every_position() {
        for size in 0..100u32 {
            for pos in 0..=size {
                let mut tree = Tree::new();
                let mut reference = BTreeSet::new();
                for i in 0..size {
                    tree.insert(k(i * 2));
                    reference.insert(i * 2);
                }
                tree.insert(k(pos * 2 + 1));
                reference.insert(pos * 2 + 1);
                verify(&tree, &reference);
            }
        }
    }

    #[test]
    fn delete_at_every_position() {
        for size in 1..100u32 {
            for pos in 0..size {
                let mut tree = Tree::new();
                let mut reference = BTreeSet::new();
                for i in 0..size {
                    tree.insert(k(i));
                    reference.insert(i);
                }
                tree.delete(k(pos));
                reference.remove(&pos);
                verify(&tree, &reference);
            }
        }
    }

    #[test]
    fn random_insert_delete() {
        let mut tree = Tree::new();
        let mut reference = BTreeSet::new();
        let mut x: u32 = 123456789;
        for _ in 0..10_000 {
            x ^= x << 13;
            x ^= x >> 17;
            x ^= x << 5;
            let n = x % 1000;
            if x % 3 == 0 {
                tree.delete(k(n));
                reference.remove(&n);
            } else {
                tree.insert(k(n));
                reference.insert(n);
            }
            verify(&tree, &reference);
        }
    }
}

fn rank(node: &Option<Box<Node>>, key: &Key) -> u32 {
    match node {
        None => 0,
        Some(val) => match key.cmp(&val.key) {
            Ordering::Less => rank(&val.left, key),
            Ordering::Greater => count(&val.left) + 1 + rank(&val.right, key),
            Ordering::Equal => count(&val.left),
        },
    }
}

fn select(node: &Option<Box<Node>>, n: u32) -> Option<&Key> {
    let val = node.as_ref()?;
    let left = count(&val.left);
    match n.cmp(&left) {
        Ordering::Less => select(&val.left, n),
        Ordering::Equal => Some(&val.key),
        Ordering::Greater => select(&val.right, n - left - 1),
    }
}
