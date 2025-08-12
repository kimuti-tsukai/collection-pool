pub mod sync;

use innermut::InnerMut;

use std::cell::RefCell;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::ops::{Deref, DerefMut};

/// Trait for types that can be retrieved from the pool
pub trait Clearable {
    fn clear(&mut self);
}

impl<T> Clearable for Vec<T> {
    fn clear(&mut self) {
        self.clear();
    }
}

impl<K, V> Clearable for HashMap<K, V> {
    fn clear(&mut self) {
        self.clear();
    }
}

impl<T> Clearable for HashSet<T> {
    fn clear(&mut self) {
        self.clear();
    }
}

impl Clearable for String {
    fn clear(&mut self) {
        self.clear();
    }
}

impl<T> Clearable for VecDeque<T> {
    fn clear(&mut self) {
        self.clear();
    }
}

impl<T: Ord> Clearable for BinaryHeap<T> {
    fn clear(&mut self) {
        self.clear();
    }
}

/// Generic object pool
pub struct Pool<T: Clearable, M: InnerMut<Inner = Vec<T>>> {
    vacant: M,
}

/// Object borrowed from the pool
pub struct Pooled<'a, T: Clearable, M: InnerMut<Inner = Vec<T>>> {
    item: Option<T>,
    parent: &'a Pool<T, M>,
}

impl<'a, T: Clearable, M: InnerMut<Inner = Vec<T>>> Deref for Pooled<'a, T, M> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.item.as_ref().unwrap()
    }
}

impl<'a, T: Clearable, M: InnerMut<Inner = Vec<T>>> DerefMut for Pooled<'a, T, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.item.as_mut().unwrap()
    }
}

impl<'a, T: Clearable, M: InnerMut<Inner = Vec<T>>> Drop for Pooled<'a, T, M> {
    fn drop(&mut self) {
        let Some(mut item) = self.item.take() else {
            return;
        };

        item.clear();

        if let Ok(mut vacant) = self.parent.vacant.inner_mut() {
            vacant.push(item);
        }
        // If borrow fails, the item is destroyed (prevents memory leak)
    }
}

impl<T: Clearable + Default, M: InnerMut<Inner = Vec<T>> + Default> Pool<T, M> {
    pub fn new() -> Self {
        Self {
            vacant: M::default(),
        }
    }

    pub fn get(&self) -> Pooled<'_, T, M> {
        let mut item = self
            .vacant
            .inner_mut()
            .ok()
            .and_then(|mut v| v.pop())
            .unwrap_or_default();

        item.clear(); // Clear again just to be safe

        Pooled {
            item: Some(item),
            parent: self,
        }
    }

    pub fn prewarm(&self, count: usize) -> Result<(), <M as InnerMut>::Error<'_>> {
        let mut vacant = self.vacant.inner_mut()?;
        for _ in 0..count {
            vacant.push(T::default());
        }
        Ok(())
    }

    /// Get the number of objects in the pool
    pub fn pool_size(&self) -> Option<usize> {
        self.vacant.inner().ok().map(|v| v.len())
    }
}

impl<T: Clearable + Default, M: InnerMut<Inner = Vec<T>> + Default> Default for Pool<T, M> {
    fn default() -> Self {
        Self::new()
    }
}

type RefPool<T> = Pool<T, RefCell<Vec<T>>>;

// Convenient alias types
pub type VecPool<T> = RefPool<Vec<T>>;
pub type HashMapPool<K, V> = RefPool<HashMap<K, V>>;
pub type HashSetPool<T> = RefPool<HashSet<T>>;
pub type StringPool = RefPool<String>;
pub type VecDequePool<T> = RefPool<VecDeque<T>>;
pub type BinaryHeapPool<T> = RefPool<BinaryHeap<T>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vec_pool() {
        let pool: VecPool<i32> = VecPool::new();

        {
            let mut vec = pool.get();
            vec.push(1);
            vec.push(2);
            vec.push(3);
            assert_eq!(&*vec, &vec![1, 2, 3]);
        }

        assert_eq!(pool.pool_size().unwrap(), 1);

        {
            let vec = pool.get();
            assert_eq!(vec.len(), 0); // Cleared
        }
    }

    #[test]
    fn test_hashmap_pool() {
        let pool: HashMapPool<String, i32> = HashMapPool::new();

        {
            let mut map = pool.get();
            map.insert("key1".to_string(), 42);
            map.insert("key2".to_string(), 84);
            assert_eq!(map.len(), 2);
            assert_eq!(map["key1"], 42);
        }

        assert_eq!(pool.pool_size().unwrap(), 1);

        {
            let map = pool.get();
            assert_eq!(map.len(), 0); // Cleared
        }
    }

    #[test]
    fn test_string_pool() {
        let pool: StringPool = StringPool::new();

        {
            let mut s = pool.get();
            s.push_str("Hello, ");
            s.push_str("World!");
            assert_eq!(&*s, "Hello, World!");
        }

        assert_eq!(pool.pool_size().unwrap(), 1);

        {
            let s = pool.get();
            assert_eq!(s.len(), 0); // Cleared
        }
    }

    #[test]
    fn test_hashset_pool() {
        let pool: HashSetPool<i32> = HashSetPool::new();

        {
            let mut set = pool.get();
            set.insert(1);
            set.insert(2);
            set.insert(3);
            assert_eq!(set.len(), 3);
            assert!(set.contains(&2));
        }

        assert_eq!(pool.pool_size().unwrap(), 1);

        {
            let set = pool.get();
            assert_eq!(set.len(), 0); // Cleared
        }
    }

    #[test]
    fn test_vecdeque_pool() {
        let pool: VecDequePool<i32> = VecDequePool::new();

        {
            let mut deque = pool.get();
            deque.push_back(1);
            deque.push_front(0);
            deque.push_back(2);
            assert_eq!(deque.len(), 3);
            assert_eq!(deque[0], 0);
            assert_eq!(deque[1], 1);
            assert_eq!(deque[2], 2);
        }

        assert_eq!(pool.pool_size().unwrap(), 1);

        {
            let deque = pool.get();
            assert_eq!(deque.len(), 0); // Cleared
        }
    }

    #[test]
    fn test_multiple_objects() {
        let pool: VecPool<i32> = VecPool::new();

        // Get multiple objects simultaneously
        {
            let mut vec1 = pool.get();
            let mut vec2 = pool.get();
            let mut vec3 = pool.get();

            vec1.push(1);
            vec2.extend(vec![2, 20]);
            vec3.extend(vec![3, 30, 300]);

            assert_eq!(&*vec1, &vec![1]);
            assert_eq!(&*vec2, &vec![2, 20]);
            assert_eq!(&*vec3, &vec![3, 30, 300]);

            assert_eq!(pool.pool_size().unwrap(), 0); // All in use
        }

        // All three are returned to the pool
        assert_eq!(pool.pool_size().unwrap(), 3);
    }

    #[test]
    fn test_capacity_preservation() {
        let pool: VecPool<i32> = VecPool::new();

        {
            let mut vec = pool.get();
            vec.reserve(1000);
            vec.extend(0..100);
            assert!(vec.capacity() >= 1000);
        }

        {
            let vec = pool.get();
            assert_eq!(vec.len(), 0); // Cleared
            assert!(vec.capacity() >= 1000); // Capacity is preserved
        }
    }
}
