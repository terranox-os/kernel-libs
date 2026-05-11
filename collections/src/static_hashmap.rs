//! Array-backed hash map with fixed capacity.
//!
//! Uses open addressing with linear probing. No heap allocation.
//! Keys must implement `Eq` and provide a hash via the `Hashable` trait.

/// Trait for types that can produce a hash value.
/// We define our own instead of depending on `core::hash` to keep
/// the implementation minimal and avoid needing a `Hasher`.
pub trait Hashable {
    fn hash(&self) -> u64;
}

impl Hashable for u32 {
    fn hash(&self) -> u64 {
        // FNV-1a inspired mixing
        let mut h = 0xcbf29ce484222325u64;
        let bytes = self.to_le_bytes();
        for &b in &bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }
}

impl Hashable for u64 {
    fn hash(&self) -> u64 {
        let mut h = 0xcbf29ce484222325u64;
        let bytes = self.to_le_bytes();
        for &b in &bytes {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }
}

impl Hashable for &str {
    fn hash(&self) -> u64 {
        let mut h = 0xcbf29ce484222325u64;
        for &b in self.as_bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h
    }
}

enum Slot<K, V> {
    Empty,
    Occupied(K, V),
    Tombstone,
}

/// A fixed-capacity hash map using open addressing with linear probing.
pub struct StaticHashMap<K, V, const N: usize> {
    slots: [Slot<K, V>; N],
    len: usize,
}

impl<K: Eq + Hashable, V, const N: usize> Default for StaticHashMap<K, V, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Eq + Hashable, V, const N: usize> StaticHashMap<K, V, N> {
    /// Create an empty hash map.
    ///
    /// # Panics
    /// Panics if N is 0.
    pub fn new() -> Self {
        assert!(N > 0, "StaticHashMap capacity must be > 0");
        Self {
            slots: core::array::from_fn(|_| Slot::Empty),
            len: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn capacity(&self) -> usize {
        N
    }

    /// Insert a key-value pair. Returns the old value if the key existed.
    /// Returns `Err((key, value))` if the map is full and the key is new.
    pub fn insert(&mut self, key: K, value: V) -> Result<Option<V>, (K, V)> {
        let start = (key.hash() as usize) % N;
        let mut first_tombstone: Option<usize> = None;

        for i in 0..N {
            let idx = (start + i) % N;
            match &self.slots[idx] {
                Slot::Occupied(k, _) if *k == key => {
                    // Replace existing
                    let old = core::mem::replace(&mut self.slots[idx], Slot::Occupied(key, value));
                    if let Slot::Occupied(_, v) = old {
                        return Ok(Some(v));
                    }
                    unreachable!();
                }
                Slot::Empty => {
                    let target = first_tombstone.unwrap_or(idx);
                    self.slots[target] = Slot::Occupied(key, value);
                    self.len += 1;
                    return Ok(None);
                }
                Slot::Tombstone if first_tombstone.is_none() => {
                    first_tombstone = Some(idx);
                }
                _ => {}
            }
        }

        // If we found a tombstone during traversal, use it
        if let Some(idx) = first_tombstone {
            self.slots[idx] = Slot::Occupied(key, value);
            self.len += 1;
            return Ok(None);
        }

        Err((key, value))
    }

    /// Get a reference to the value for `key`.
    pub fn get(&self, key: &K) -> Option<&V> {
        let start = (key.hash() as usize) % N;
        for i in 0..N {
            let idx = (start + i) % N;
            match &self.slots[idx] {
                Slot::Occupied(k, v) if *k == *key => return Some(v),
                Slot::Empty => return None,
                _ => {}
            }
        }
        None
    }

    /// Get a mutable reference to the value for `key`.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let start = (key.hash() as usize) % N;
        // Find the index first to satisfy the borrow checker
        let mut found_idx = None;
        for i in 0..N {
            let idx = (start + i) % N;
            match &self.slots[idx] {
                Slot::Occupied(k, _) if *k == *key => {
                    found_idx = Some(idx);
                    break;
                }
                Slot::Empty => break,
                _ => {}
            }
        }
        if let Some(idx) = found_idx {
            if let Slot::Occupied(_, v) = &mut self.slots[idx] {
                return Some(v);
            }
        }
        None
    }

    /// Remove a key. Returns the value if it existed.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let start = (key.hash() as usize) % N;
        for i in 0..N {
            let idx = (start + i) % N;
            match &self.slots[idx] {
                Slot::Occupied(k, _) if *k == *key => {
                    let old = core::mem::replace(&mut self.slots[idx], Slot::Tombstone);
                    self.len -= 1;
                    if let Slot::Occupied(_, v) = old {
                        return Some(v);
                    }
                    unreachable!();
                }
                Slot::Empty => return None,
                _ => {}
            }
        }
        None
    }

    /// Check if the map contains `key`.
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_get() {
        let mut map: StaticHashMap<u32, u32, 16> = StaticHashMap::new();
        assert!(map.insert(1, 100).unwrap().is_none());
        assert!(map.insert(2, 200).unwrap().is_none());
        assert_eq!(map.get(&1), Some(&100));
        assert_eq!(map.get(&2), Some(&200));
        assert_eq!(map.get(&3), None);
    }

    #[test]
    fn test_overwrite() {
        let mut map: StaticHashMap<u32, u32, 16> = StaticHashMap::new();
        map.insert(1, 100).unwrap();
        let old = map.insert(1, 999).unwrap();
        assert_eq!(old, Some(100));
        assert_eq!(map.get(&1), Some(&999));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_remove() {
        let mut map: StaticHashMap<u32, u32, 16> = StaticHashMap::new();
        map.insert(1, 100).unwrap();
        map.insert(2, 200).unwrap();

        assert_eq!(map.remove(&1), Some(100));
        assert_eq!(map.get(&1), None);
        assert_eq!(map.len(), 1);
        assert_eq!(map.remove(&1), None);
    }

    #[test]
    fn test_full_map() {
        let mut map: StaticHashMap<u32, u32, 4> = StaticHashMap::new();
        map.insert(1, 10).unwrap();
        map.insert(2, 20).unwrap();
        map.insert(3, 30).unwrap();
        map.insert(4, 40).unwrap();
        assert_eq!(map.len(), 4);
        assert!(map.insert(5, 50).is_err());
    }

    #[test]
    fn test_tombstone_reuse() {
        let mut map: StaticHashMap<u32, u32, 4> = StaticHashMap::new();
        map.insert(1, 10).unwrap();
        map.insert(2, 20).unwrap();
        map.insert(3, 30).unwrap();
        map.insert(4, 40).unwrap();

        map.remove(&2);
        assert_eq!(map.len(), 3);

        // Should reuse the tombstone slot
        assert!(map.insert(5, 50).is_ok());
        assert_eq!(map.get(&5), Some(&50));
    }

    #[test]
    fn test_contains_key() {
        let mut map: StaticHashMap<u32, u32, 8> = StaticHashMap::new();
        map.insert(42, 0).unwrap();
        assert!(map.contains_key(&42));
        assert!(!map.contains_key(&99));
    }

    #[test]
    fn test_get_mut() {
        let mut map: StaticHashMap<u32, u32, 8> = StaticHashMap::new();
        map.insert(1, 10).unwrap();
        if let Some(v) = map.get_mut(&1) {
            *v = 99;
        }
        assert_eq!(map.get(&1), Some(&99));
    }
}
