use std::collections::HashMap;

use crate::{avl::Tree, key::Key};

pub const LARGE_ZSET: usize = 1000;

#[derive(Debug)]
pub struct ZSet {
    scores: HashMap<String, f64>,
    tree: Tree,
}

impl ZSet {
    pub fn new() -> Self {
        Self {
            scores: HashMap::new(),
            tree: Tree::new(),
        }
    }
    pub fn add(&mut self, name: &str, score: f64) -> bool {
        match self.scores.get(name) {
            Some(&old) if old == score => false,
            Some(&old) => {
                self.tree.delete(Key::new(old, name));
                self.tree.insert(Key::new(score, name));
                self.scores.insert(name.to_string(), score);
                false
            }
            None => {
                self.tree.insert(Key::new(score, name));
                self.scores.insert(name.to_string(), score);
                true
            }
        }
    }

    pub fn remove(&mut self, name: &str) -> bool {
        match self.scores.remove(name) {
            Some(score) => {
                self.tree.delete(Key::new(score, name));
                true
            }
            None => false,
        }
    }

    pub fn score(&self, name: &str) -> Option<f64> {
        self.scores.get(name).copied()
    }

    pub fn query(&self, score: f64, name: &str, offset: u32, limit: u32) -> Vec<&Key> {
        self.tree.range(&Key::new(score, name), offset, limit)
    }
    pub fn len(&self) -> usize {
        self.scores.len()
    }
}
