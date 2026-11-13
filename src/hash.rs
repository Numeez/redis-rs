use std::vec;

const P: u64 = 0x100000001b3u64;
const MAX_LOAD_FACTOR: usize = 8;

pub struct HMap {
    newer: Table,
    older: Table,
    migration_pos: usize,
}

impl HMap {
    pub fn new(n: usize) -> Self {
        Self {
            newer: Table::new(n),
            older: Table::empty(),
            migration_pos: 0,
        }
    }
    pub fn look_up(&mut self, key: &[u8]) -> Option<&Vec<u8>> {
        self.rehashing();
        let hash = hash(key);
        let mut result = self.newer.lookup(hash, key);
        if result.is_none() {
            result = self.older.lookup(hash, key);
            if result.is_none() {
                return None;
            }
        }
        Some(&result.unwrap().val)
    }
    pub fn insert(&mut self, key: Vec<u8>, val: Vec<u8>) {
        self.rehashing();
        let hash = hash(&key);
        if let Some(value) = self.newer.lookup_mut(hash, &key) {
            *value = val;
            return;
        }
        if let Some(value) = self.older.lookup_mut(hash, &key) {
            *value = val;
            return;
        }
        self.newer.insert(Box::new(Node {
            h_code: hash,
            key,
            val,
            next: None,
        }));

        if self.older.len() == 0 && self.newer.len() >= self.newer.capacity() * MAX_LOAD_FACTOR {
            let new_capacity = self.newer.capacity() * 2;
            self.older = std::mem::replace(&mut self.newer, Table::new(new_capacity));
            self.migration_pos = 0;
        }
    }
    fn rehashing(&mut self) {
        let mut counter = 0;
        while counter < 128 && self.older.len() > 0 {
            let node = self.older.pop_from_slot(self.migration_pos);
            match node {
                Some(val) => {
                    self.newer.insert(val);
                    counter += 1;
                }
                None => self.migration_pos += 1,
            }
        }
        if self.older.len() == 0 && self.older.capacity() > 0 {
            self.older = Table::empty()
        }
    }
    pub fn remove(&mut self, key: &[u8]) -> bool {
        self.rehashing();
        let hash = hash(key);
        if self.newer.remove(hash, key).is_some() {
            return true;
        }
        if self.older.remove(hash, key).is_some() {
            return true;
        }
        false
    }

    pub fn len(&self) -> usize {
        self.newer.len() + self.older.len()
    }
    pub fn keys(&self) -> Vec<&[u8]> {
        let mut keys: Vec<&[u8]> = Vec::with_capacity(self.len());
        self.newer.collect(&mut keys);
        self.older.collect(&mut keys);
        keys
    }
}

struct Table {
    slots: Vec<Option<Box<Node>>>,
    mask: usize,
    size: usize,
}
impl Table {
    pub fn new(n: usize) -> Self {
        assert!(n.is_power_of_two());
        Self {
            slots: (0..n).map(|_| None).collect(),
            mask: n - 1,
            size: 0,
        }
    }
    pub fn empty() -> Self {
        Self {
            slots: vec![],
            mask: 0,
            size: 0,
        }
    }
    pub fn insert(&mut self, mut node: Box<Node>) {
        let i = node.h_code as usize & self.mask;
        node.next = self.slots[i].take();
        self.slots[i] = Some(node);
        self.size += 1;
    }
    pub fn lookup(&self, hash_code: u64, key: &[u8]) -> Option<&Node> {
        if self.slots.is_empty() {
            return None;
        }
        let i = hash_code as usize & self.mask;
        let mut cur = &self.slots[i];
        while let Some(node) = cur {
            if node.h_code == hash_code && node.key == key {
                return Some(node);
            }
            cur = &node.next;
        }
        None
    }
    pub fn lookup_mut(&mut self, hash_code: u64, key: &[u8]) -> Option<&mut Vec<u8>> {
        if self.slots.is_empty() {
            return None;
        }
        let i = hash_code as usize & self.mask;
        let mut cur = &mut self.slots[i];
        while let Some(node) = cur {
            if node.h_code == hash_code && node.key == key {
                return Some(&mut node.val);
            }
            cur = &mut node.next;
        }

        None
    }

    pub fn remove(&mut self, hash_code: u64, key: &[u8]) -> Option<Box<Node>> {
        if self.slots.is_empty() {
            return None;
        }
        let i = hash_code as usize & self.mask;
        let mut cur = &mut self.slots[i];
        loop {
            match cur {
                None => return None,
                Some(val) if val.h_code == hash_code && val.key == key => {
                    let mut removed = cur.take().unwrap();
                    *cur = removed.next.take();
                    self.size -= 1;
                    return Some(removed);
                }
                Some(val) => cur = &mut val.next,
            }
        }
    }

    fn pop_from_slot(&mut self, pos: usize) -> Option<Box<Node>> {
        assert!(
            pos < self.slots.len(),
            "migration position past end of table"
        );
        let mut node = self.slots[pos].take()?;
        self.slots[pos] = node.next.take();
        self.size -= 1;
        Some(node)
    }
    pub fn len(&self) -> usize {
        self.size
    }

    pub fn capacity(&self) -> usize {
        self.slots.iter().len()
    }
    pub fn collect<'a>(&'a self, out: &mut Vec<&'a [u8]>) {
        for slot in &self.slots {
            let mut cur = slot;
            while let Some(node) = cur {
                out.push(&node.key);
                cur = &node.next;
            }
        }
    }
}

struct Node {
    h_code: u64,
    key: Vec<u8>,
    val: Vec<u8>,
    next: Option<Box<Node>>,
}

pub fn hash(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes.iter() {
        h ^= *b as u64;
        h = h.wrapping_mul(P);
    }

    h
}
