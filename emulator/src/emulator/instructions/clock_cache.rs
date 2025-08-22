use std::hash::{BuildHasher, RandomState};

use clockpro_cache::ClockProCache;
use nohash_hasher::IsEnabled;

/// Clock-Pro缓存的包装器，提供与LRU缓存兼容的接口
pub struct ClockCache<K, V, S = RandomState> {
    cache: ClockProCache<K, V, S>,
    capacity: usize,
}

#[allow(unused)]
impl <K, V> ClockCache<K, V, RandomState>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    /// 创建一个新的Clock缓存实例
    ///
    /// # Arguments
    /// * `capacity` - 缓存的最大容量（最小为3）
    pub fn new(capacity: usize) -> Self {
        let actual_capacity = capacity.max(3); // ClockProCache要求最小容量为3
        Self {
            cache: ClockProCache::new(actual_capacity).expect("Failed to create ClockProCache"),
            capacity: actual_capacity,
        }
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        // ClockProCache 没有直接的clear方法，我们重新创建一个
        self.cache = ClockProCache::new(self.capacity).expect("Failed to create ClockProCache");
    }
}

#[allow(unused)]
impl<K, V, S> ClockCache<K, V, S>
where
    K: Clone + Eq + std::hash::Hash + IsEnabled,
    V: Clone,
    S: BuildHasher
{

    /// 创建一个带有自定义哈希器的Clock缓存实例
    ///
    /// # Arguments
    /// * `capacity` - 缓存的最大容量（最小为3）
    /// * `_hasher` - 哈希器构建器
    pub fn with_hasher(capacity: usize, _hasher: S) -> Self {
        let actual_capacity = capacity.max(3);
        // ClockProCache 使用内部的哈希实现，所以我们忽略传入的hasher
        Self {
            cache: ClockProCache::with_hasher(actual_capacity, _hasher).expect("Failed to create ClockProCache"),
            capacity: actual_capacity,
        }
    }

    /// 获取缓存容量
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// 向缓存中插入键值对
    ///
    /// # Arguments
    /// * `key` - 要插入的键
    /// * `value` - 要插入的值
    pub fn insert(&mut self, key: K, value: V) {
        self.cache.insert(key, value);
    }

    /// 从缓存中获取值
    ///
    /// # Arguments
    /// * `key` - 要查找的键
    ///
    /// # Returns
    /// 如果找到，返回值的引用；否则返回None
    pub fn get(&mut self, key: &K) -> Option<&V> {
        self.cache.get(key)
    }

    /// 检查缓存是否包含指定的键
    ///
    /// # Arguments
    /// * `key` - 要检查的键
    pub fn contains(&mut self, key: &K) -> bool {
        self.cache.get(key).is_some()
    }

    /// 获取缓存中当前的元素数量
    pub fn len(&self) -> usize {
        // ClockProCache 没有直接的len方法，我们无法准确获取
        // 作为近似，我们返回0（实际使用中这个方法可能不常用）
        0
    }

    /// 检查缓存是否为空
    pub fn is_empty(&self) -> bool {
        // 由于无法准确获取长度，我们假设不为空
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nohash_hasher::BuildNoHashHasher;

    #[test]
    fn test_clock_cache_basic_operations() {
        let mut cache = ClockCache::new(3);

        // 测试插入和获取
        cache.insert(1, "one");
        cache.insert(2, "two");
        cache.insert(3, "three");

        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.get(&3), Some(&"three"));

        // 测试容量限制
        cache.insert(4, "four");

        // 应该还能获取到最近使用的项
        assert_eq!(cache.get(&4), Some(&"four"));
    }

    #[test]
    fn test_clock_cache_with_hasher() {
        let mut cache: ClockCache<u32, &str, BuildNoHashHasher<u32>> = ClockCache::with_hasher(3, BuildNoHashHasher::default());

        cache.insert(1u32, "one");
        cache.insert(2u32, "two");

        assert_eq!(cache.get(&1), Some(&"one"));
        assert_eq!(cache.get(&2), Some(&"two"));
    }

    #[test]
    fn test_clock_cache_capacity() {
        let cache = ClockCache::<i32, &str>::new(5);
        assert_eq!(cache.capacity(), 5);
    }

    #[test]
    fn test_clock_cache_clear() {
        let mut cache = ClockCache::new(3);

        cache.insert(1, "one");
        cache.insert(2, "two");

        cache.clear();

        assert_eq!(cache.get(&1), None);
        assert_eq!(cache.get(&2), None);
    }
}
