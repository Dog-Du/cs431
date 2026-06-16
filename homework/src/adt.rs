use crossbeam_epoch::Guard;

/// Trait for a concurrent key-value map.
/// 用于并发键值映射的特性。
pub trait ConcurrentMap<K: ?Sized, V> {
    /// Lookups the given key to get the reference to its value.
    /// 查找给定的键以获取其对应值的引用。
    fn lookup<'a>(&'a self, key: &K, guard: &'a Guard) -> Option<&'a V>;

    /// Inserts a key-value pair.
    /// 插入一个键值对。
    fn insert(&self, key: K, value: V, guard: &Guard) -> Result<(), V>;

    /// Deletes the given key and returns a reference to its value.
    /// 删除给定的键并返回其值的引用。
    ///
    /// Unlike stack or queue's pop that can return `Option<V>`, since a `delete`d
    /// 与栈或队列的 pop 可以返回 `Option<V>` 不同，因为 `delete`d
    /// value may also be `lookup`ed, we can only return a reference, not full ownership.
    /// 值也可能被 `lookup`，我们只能返回一个引用，而不是完整的所有权。
    fn delete<'a>(&'a self, key: &K, guard: &'a Guard) -> Result<&'a V, ()>;
}

/// Trait for a concurrent set.
/// 用于并发集合的特性。
pub trait ConcurrentSet<T> {
    /// Returns `true` iff the set contains the value.
    /// 如果集合包含该值，则返回 `true`。
    fn contains(&self, value: &T) -> bool;

    /// Adds the value to the set. Returns whether the value was newly inserted.
    /// 将该值添加到集合中。返回该值是否是新插入的。
    fn insert(&self, value: T) -> bool;

    /// Removes the value from the set. Returns whether the value was present in the set.
    /// 从集合中移除该值。返回该值是否存在于集合中。
    fn remove(&self, value: &T) -> bool;
}
