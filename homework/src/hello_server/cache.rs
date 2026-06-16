//! Thread-safe key/value cache.
//! 线程安全的键/值缓存。

use std::collections::hash_map::{Entry, HashMap};
use std::hash::Hash;
use std::sync::{Arc, Mutex, RwLock};

/// Cache that remembers the result for each key.
/// 缓存可以记住每个键的结果。
#[derive(Debug)]
pub struct Cache<K, V> {
    // todo! This is an example cache type. Build your own cache type that satisfies the
    // 待办！这是一个示例缓存类型。构建你自己的满足要求的缓存类型
    // specification for `get_or_insert_with`.
    // `get_or_insert_with`的规格。
    inner: Mutex<HashMap<K, V>>,
}

impl<K, V> Default for Cache<K, V> {
    fn default() -> Self {
        Self {
            inner: Mutex::new(HashMap::new()),
        }
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Cache<K, V> {
    /// Retrieve the value or insert a new one created by `f`.
    /// 检索该值或插入由 `f` 创建的新值。
    ///
    /// An invocation to this function should not block another invocation with a different key. For
    /// 对这个函数的调用不应阻塞使用不同键的另一次调用。对于
    /// example, if a thread calls `get_or_insert_with(key1, f1)` and another thread calls
    /// 例如，如果一个线程调用 `get_or_insert_with(key1, f1)`，而另一个线程调用
    /// `get_or_insert_with(key2, f2)` (`key1≠key2`, `key1,key2∉cache`) concurrently, `f1` and `f2`
    /// `get_or_insert_with(key2, f2)`（`key1≠key2`、`key1,key2∉cache`）同时，`f1` 和 `f2`
    /// should run concurrently.
    /// 应该同时运行。
    ///
    /// On the other hand, since `f` may consume a lot of resource (= money), it's undesirable to
    /// 另一方面，由于 `f` 可能消耗大量资源（=金钱），这是不希望的
    /// duplicate the work. That is, `f` should be run only once for each key. Specifically, even
    /// 重复工作。也就是说，`f` 应该对每个密钥只运行一次。具体来说，即使
    /// for concurrent invocations of `get_or_insert_with(key, f)`, `f` is called only once per key.
    /// 对于 `get_or_insert_with(key, f)` 的并发调用，`f` 每个键仅调用一次。
    ///
    /// Hint: the [`Entry`] API may be useful in implementing this function.
    /// 提示：[`Entry`] API 可能在实现此功能时有用。
    ///
    /// [`Entry`]: https://doc.rust-lang.org/stable/std/collections/hash_map/struct.HashMap.html#method.entry
    pub fn get_or_insert_with<F: FnOnce(K) -> V>(&self, key: K, f: F) -> V {
    }
}
