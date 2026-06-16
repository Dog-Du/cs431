//! Split-ordered linked list.
//! 分裂有序链表。

use core::mem::{self, MaybeUninit};
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering::*;

use crossbeam_epoch::{Guard, Owned};
use cs431::lockfree::list::{Cursor, List, Node};

use super::growable_array::GrowableArray;
use crate::ConcurrentMap;

/// Lock-free map from `usize` in range \[0, 2^63-1\] to `V`.
/// 无锁地图从 `usize` 在范围 [0, 2^63-1] 到 `V`。
///
/// NOTE: We don't care about hashing in this homework for simplicity.
/// 注意：为了简单起见，我们在这个作业中不关心哈希。
#[derive(Debug)]
pub struct SplitOrderedList<V> {
    /// Lock-free list sorted by recursive-split order.
    /// 按递归拆分顺序排序的无锁列表。
    ///
    /// Use `MaybeUninit::uninit()` when creating sentinel nodes.
    /// 在创建哨兵节点时使用 `MaybeUninit::uninit()`。
    list: List<usize, MaybeUninit<V>>,
    /// Array of pointers to the buckets.
    /// 指向桶的指针数组。
    buckets: GrowableArray<Node<usize, MaybeUninit<V>>>,
    /// Number of buckets.
    /// 桶的数量。
    size: AtomicUsize,
    /// Number of items.
    /// 物品数量。
    count: AtomicUsize,
}

impl<V> Default for SplitOrderedList<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> SplitOrderedList<V> {
    /// `size` is doubled when `count > size * LOAD_FACTOR`.
    /// 当 `count > size * LOAD_FACTOR` 时，`size` 会加倍。
    const LOAD_FACTOR: usize = 2;

    /// Creates a new split ordered list.
    /// 创建一个新的分割有序列表。
    pub fn new() -> Self {
        Self {
            list: List::new(),
            buckets: GrowableArray::new(),
            size: AtomicUsize::new(2),
            count: AtomicUsize::new(0),
        }
    }

    /// Creates a cursor and moves it to the bucket for the given index.  If the bucket doesn't
    /// 创建一个cursor并将其移动到给定索引的桶。如果桶不存在
    /// exist, recursively initializes the buckets.
    /// 存在，递归初始化桶。
    fn lookup_bucket<'s>(
        &'s self,
        index: usize,
        guard: &'s Guard,
    ) -> Cursor<'s, usize, MaybeUninit<V>> {
        todo!()
    }

    /// Moves the bucket cursor returned from `lookup_bucket` to the position of the given key.
    /// 将从 `lookup_bucket` 返回的桶cursor移动到给定键的位置。
    /// Returns `(size, found, cursor)`
    /// 返回 `(size, found, cursor)`
    fn find<'s>(
        &'s self,
        key: &usize,
        guard: &'s Guard,
    ) -> (usize, bool, Cursor<'s, usize, MaybeUninit<V>>) {
        todo!()
    }

    fn assert_valid_key(key: usize) {
        assert!(key.leading_zeros() != 0);
    }
}

impl<V> ConcurrentMap<usize, V> for SplitOrderedList<V> {
    fn lookup<'a>(&'a self, key: &usize, guard: &'a Guard) -> Option<&'a V> {
        Self::assert_valid_key(*key);

        todo!()
    }

    fn insert(&self, key: usize, value: V, guard: &Guard) -> Result<(), V> {
        Self::assert_valid_key(key);

        todo!()
    }

    fn delete<'a>(&'a self, key: &usize, guard: &'a Guard) -> Result<&'a V, ()> {
        Self::assert_valid_key(*key);

        todo!()
    }
}
