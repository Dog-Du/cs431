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
        let bucket = self.buckets.get(index, guard);
        let initialized = bucket.load(SeqCst, guard);

        // 已初始化的桶直接从哨兵节点开始遍历。GrowableArray 只保存借用指针，
        // 节点本身始终由 list 持有，因此这里不会产生重复所有权。
        if !initialized.is_null() {
            return Cursor::new(bucket, initialized);
        }

        // 除 0 号桶外，每个桶的父桶都是“清除桶编号最高有效位”后的桶。
        // 例如 0b1101 的父桶是 0b0101。先初始化父桶，才能从父桶哨兵
        // 向后寻找并插入当前桶的哨兵。
        let parent_index = if index == 0 {
            None
        } else {
            let highest_bit =
                1 << (mem::size_of::<usize>() * 8 - index.leading_zeros() as usize - 1);
            Some(index & !highest_bit)
        };

        // 桶哨兵使用反转后的桶编号，最低位一定是 0；真实键的编码最低位
        // 一定是 1，因此二者不会冲突。哨兵不存放 V，value 保持未初始化。
        let sentinel_key = index.reverse_bits();
        let mut sentinel = Owned::new(Node::new(sentinel_key, MaybeUninit::uninit()));

        let sentinel_ptr = loop {
            let mut cursor = match parent_index {
                Some(parent) => self.lookup_bucket(parent, guard),
                None => self.list.head(guard),
            };

            // Harris 查找可能因并发清理失败；此时从稳定的父桶位置重新开始。
            match cursor.find_harris(&sentinel_key, guard) {
                Ok(true) => break cursor.curr(),
                Ok(false) => match cursor.insert(sentinel, guard) {
                    Ok(()) => break cursor.curr(),
                    Err(node) => sentinel = node,
                },
                Err(()) => {}
            }
        };

        // 多个线程可能同时找到或插入同一个哨兵。只有一个线程负责把它发布到
        // 桶数组；失败者使用胜出线程已经写入的同一个哨兵指针。
        let initialized = match bucket.compare_exchange(
            crossbeam_epoch::Shared::null(),
            sentinel_ptr,
            SeqCst,
            SeqCst,
            guard,
        ) {
            Ok(installed) => installed,
            Err(error) => error.current,
        };

        Cursor::new(bucket, initialized)
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
        // size 始终是 2 的幂，所以低位掩码可以直接得到桶编号。
        let size = self.size.load(SeqCst);
        let bucket_index = key & (size - 1);
        let ordered_key = key.reverse_bits() | 1;

        loop {
            let mut cursor = self.lookup_bucket(bucket_index, guard);
            if let Ok(found) = cursor.find_harris(&ordered_key, guard) {
                return (size, found, cursor);
            }
            // 清理已逻辑删除节点时发生竞争，需要从桶哨兵重新查找。
        }
    }

    fn assert_valid_key(key: usize) {
        // 最高位必须为 0：反转后最低位才能留给“哨兵(0)/真实键(1)”标记。
        assert!(key.leading_zeros() != 0);
    }
}

impl<V> ConcurrentMap<usize, V> for SplitOrderedList<V> {
    fn lookup<'a>(&'a self, key: &usize, guard: &'a Guard) -> Option<&'a V> {
        Self::assert_valid_key(*key);

        let (_, found, cursor) = self.find(key, guard);
        if !found {
            return None;
        }

        // SAFETY: find 查找的是最低位为 1 的真实键，而所有桶哨兵的最低位为 0。
        // 因而 found 为 true 时，当前节点一定由 insert 用 MaybeUninit::new 初始化过。
        Some(unsafe { cursor.lookup().assume_init_ref() })
    }

    fn insert(&self, key: usize, value: V, guard: &Guard) -> Result<(), V> {
        Self::assert_valid_key(key);

        let ordered_key = key.reverse_bits() | 1;
        let mut node = Owned::new(Node::new(ordered_key, MaybeUninit::new(value)));

        loop {
            let (size, found, mut cursor) = self.find(&key, guard);
            if found {
                // node 从未发布，仍由当前线程独占；其中的 value 一定已初始化。
                let value = node.into_box().into_value();
                // SAFETY: 上面构造节点时使用了 MaybeUninit::new。
                return Err(unsafe { value.assume_init() });
            }

            // 在发布节点前预留计数。这样节点一旦能被其他线程删除，就已经有
            // 对应的计数，避免“插入成功但尚未计数”期间删除导致 usize 下溢。
            let count = self.count.fetch_add(1, SeqCst) + 1;

            match cursor.insert(node, guard) {
                Ok(()) => {
                    // 扩容只改变用于选择桶的掩码，不移动链表节点。并发线程可能
                    // 同时尝试扩容，compare_exchange 保证每个 size 只翻倍一次。
                    if count > size * Self::LOAD_FACTOR {
                        let _ = self.size.compare_exchange(size, size * 2, SeqCst, SeqCst);
                    }
                    return Ok(());
                }
                Err(returned) => {
                    // CAS 失败时节点没有发布，撤销预留计数并用原节点重试，
                    // 从而既不会重复构造 V，也不会提前释放它。
                    self.count.fetch_sub(1, SeqCst);
                    node = returned;
                }
            }
        }
    }

    fn delete<'a>(&'a self, key: &usize, guard: &'a Guard) -> Result<&'a V, ()> {
        Self::assert_valid_key(*key);

        loop {
            let (_, found, mut cursor) = self.find(key, guard);
            if !found {
                return Err(());
            }

            if let Ok(value) = cursor.delete(guard) {
                self.count.fetch_sub(1, SeqCst);

                // SAFETY: find 只会命中真实键节点；真实键节点的 value 由
                // insert 使用 MaybeUninit::new 初始化，且 guard 保证引用仍有效。
                return Ok(unsafe { value.assume_init_ref() });
            }
            // 另一个线程先完成了逻辑删除；重新查找以判断键是否已不存在，
            // 或者是否在此期间又插入了同名的新节点。
        }
    }
}
