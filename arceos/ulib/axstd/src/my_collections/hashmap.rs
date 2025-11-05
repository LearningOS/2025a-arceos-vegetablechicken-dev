use alloc::boxed::Box;
use alloc::vec::Vec;
use alloc::string::String;
use core::iter::Iterator;

/// The node contains key and value
struct Node<K, V> {
    pub key: K,
    pub value: V,
    pub next: Option<Box<Node<K, V>>>,
}

impl<K, V> Node<K, V> {
    /// create a node
    fn new(key: K, value: V) -> Self {
        Node {
            key,
            value,
            next: None,
        }
    }
}

/// We use separate chaining to solve hash collision
pub struct HashMap<K, V> {
    /// bucket vec
    buckets: Vec<Option<Box<Node<K, V>>>>,
    /// size of hashmap
    size: usize,
    /// the capacity of the hashmap
    capacity: usize,
}
/// hash trait
pub trait MyHash {
    fn hash(&self) -> usize;
}

/// hash implement for str
impl MyHash for str {
    fn hash(&self) -> usize {
        let mut h: usize = 5381;
        for c in self.bytes() {
            h = ((h << 5) + h) + c as usize;
        }
        h
    }
}

impl MyHash for &str {
    fn hash(&self) -> usize {
        (*self).hash()
    }
}

impl MyHash for String {
    fn hash(&self) -> usize {
        self[..].hash()
    }
}

/// implement for int
impl MyHash for i8 {
    fn hash(&self) -> usize {
        *self as usize
    }
}

impl MyHash for i16 {
    fn hash(&self) -> usize {
        *self as usize
    }
}

impl MyHash for i32 {
    fn hash(&self) -> usize {
        *self as usize
    }
}

impl MyHash for i64 {
    fn hash(&self) -> usize {
        *self as usize
    }
}

/// implement for unsigned int
impl MyHash for u8 {
    fn hash(&self) -> usize {
        *self as usize
    }
}

impl MyHash for u16 {
    fn hash(&self) -> usize {
        *self as usize
    }
}

impl MyHash for u32 {
    fn hash(&self) -> usize {
        *self as usize
    }
}

impl MyHash for u64 {
    fn hash(&self) -> usize {
        *self as usize
    }
}

/// implement for bool
impl MyHash for bool {
    fn hash(&self) -> usize {
        match *self {
            true => 1,
            false => 0,
        }
    }
}

impl<K, V> HashMap<K, V>
where
    K: MyHash + Eq + Clone,
    V: Clone,
{
    /// new a hashmap
    /// the capacity is 16 by default
    pub fn new() -> Self {
        Self::with_capacity(65_536)
    }
    /// new a hashmap and set its capacity
    // 创建新HashMap，指定桶数量（建议为质数以减少冲突）
    pub fn with_capacity(capacity: usize) -> Self {
        // The min of capacity is 1
        let capacity = capacity.max(1);
        let mut buckets = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            buckets.push(None);
        }
        HashMap {
            buckets,
            size: 0,
            capacity,
        }
    }
    /// calc load factor
    fn load_factor(&self) -> f64 {
        self.size as f64 / self.capacity as f64
    }
    /// get len of the hashmap
    pub fn len(&self) -> usize {
        self.size
    }
    /// is hashmap empty
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
    /// calc index
    fn bucket_index(&self, key: &K) -> usize {
        key.hash() % self.capacity
    }
    /// get the value by key
    /// if the key exists, return Some
    /// else return None
    /// We use Separate Chaining to resolve hash conflicts
    pub fn get(&self, key: &K) -> Option<&V> {
        let index = self.bucket_index(key);
        let mut cur = &self.buckets[index];
        while let Some(node) = cur {
            if node.key == *key {
                return Some(&node.value);
            }
            cur = &node.next;
        }
        None
    }
    /// insert into hashmap
    /// if the key exists, update the value
    /// else insert
    pub fn insert(&mut self, key: K, value: V) {
        let index = self.bucket_index(&key);
        let mut cur = &mut self.buckets[index];
        while let Some(node) = cur {
            if node.key == key {
                node.value = value.clone();
                return;
            }
            cur = &mut node.next;
        }
        let new_node = Some(Box::new(Node::new(key, value)));
        *cur = new_node;
        self.size += 1;
        if self.load_factor() > 0.7 {
            self.resize()
        }
    }
    /// remove entry
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let index = self.bucket_index(key);
        let bucket = &mut self.buckets[index];
        let mut prev = bucket;

        loop {
            match prev {
                // Not found key
                None => return None,
                // Found key, and key matches
                Some(node) if &node.key == key => {
                    // take the node, prev becomes None
                    let removed_node = prev.take().unwrap();
                    // set prev.next to removed_node.next
                    *prev = removed_node.next;
                    self.size -= 1;
                    return Some(removed_node.value);
                }

                // Found index, but key doesn't match
                // find the next node
                Some(node) => {
                    prev = &mut node.next;
                }
            }
        }
    }
    /// enlarge hashmap
    fn resize(&mut self) {
        // TODO
    }
    /// iterator for hashmap
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            buckets: &self.buckets,
            current_bucket: 0,
            current_node: None,
        }
    }
}

pub struct Iter<'a, K, V> {
    /// ref of bucket vec
    buckets: &'a [Option<Box<Node<K, V>>>],
    /// current index in bucket vec
    current_bucket: usize,
    /// current node in linked list
    current_node: Option<&'a Node<K, V>>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    /// get next element in bucket vec
    fn next(&mut self) -> Option<Self::Item> {
        // get the next field in node first
        if let Some(node) = self.current_node {
            if let Some(next_node) = &node.next {
                self.current_node = Some(next_node);
                return Some((&next_node.key, &next_node.value));
            } else {
                // if next is none, get the next bucket
                self.current_node = None;
                self.current_bucket += 1;
            }
        }

        // find a non-null bucket
        while self.current_bucket < self.buckets.len() {
            let bucket = &self.buckets[self.current_bucket];
            if let Some(node) = bucket {
                self.current_node = Some(node);
                return Some((&node.key, &node.value));
            }
            self.current_bucket += 1;
        }
        None
    }
}