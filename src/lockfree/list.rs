//! Lock-free singly linked list.

use core::cmp::Ordering::*;
use core::mem;
use core::sync::atomic::Ordering::*;

use crossbeam_epoch::{Atomic, Guard, Owned, Shared};

/// Linked list node.
/// 链表节点。
// TODO: This node type is very brittle; what if some list creates a node, and uses it to add it to
// 待办：这种节点类型非常脆弱；如果某个列表创建了一个节点，并用它来添加它会怎么样
// another, separate list? Also see the discussions at <https://github.com/kaist-cp/cs431/issues/957>.
// 另一个独立的列表？也请参阅 <https://github.com/kaist-cp/cs431/issues/957> 的讨论。
// The public API surface is way too large.
// 公共 API 面向过于庞大。
#[derive(Debug)]
pub struct Node<K, V> {
    /// Mark: tag(), Tag: not needed
    /// 标记：tag()，标签：不需要
    next: Atomic<Node<K, V>>,
    key: K,
    value: V,
}

/// Sorted singly linked list.
/// 已排序的单向链表。
///
/// Use-after-free will be caused when an unprotected guard is used, as the lifetime of returned
/// 当使用未受保护的保护时，将会导致使用后释放，因为返回的生命周期
/// elements are linked to that of the guard in the same way a [`Shared`] is.
/// 元素以与[`Shared`]相同的方式与守卫的元素相连接。
#[derive(Debug)]
pub struct List<K, V> {
    head: Atomic<Node<K, V>>,
}

// Unlike stack and queue, we need `K` and `V` to be `Sync` for the list to be `Sync`, as both `K`
// 与栈和队列不同，我们需要 `K` 和 `V` 为 `Sync`，以便列表为 `Sync`，因为两者都是 `K`
// and `V` are accessed concurrently in `find` and `delete`, respectively.
// 并且 `V` 分别在 `find` 和 `delete` 中被同时访问。
unsafe impl<K: Sync, V: Sync> Sync for List<K, V> {}
unsafe impl<K: Send, V: Send> Send for List<K, V> {}

impl<K, V> Default for List<K, V>
where
    K: Ord,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Drop for List<K, V> {
    fn drop(&mut self) {
        let mut o_curr = mem::take(&mut self.head);
        // SAFETY: since we have `&mut self`, any references from `lookup()` must have finished.
        // 安全：由于我们有 `&mut self`，任何来自 `lookup()` 的参考都必须完成。
        // Hence, we have sole ownership of `self` and its `Node`s.
        // 因此，我们拥有 `self` 及其 `Node` 的唯一所有权。
        while let Some(curr) = unsafe { o_curr.try_into_owned() }.map(Owned::into_box) {
            o_curr = curr.next;
        }
    }
}

/// Linked list cursor.
/// 链表 游标。
#[derive(Debug)]
pub struct Cursor<'g, K, V> {
    prev: &'g Atomic<Node<K, V>>,
    // Tag of `curr` should always be zero so when `curr` is stored in a `prev`, we don't store a
    // `curr` 的标签应始终为零，因此当 `curr` 存储在 `prev` 中时，我们不会存储 a
    // marked pointer and cause cleanup to fail.
    // 标记的指针并导致清理失败。
    curr: Shared<'g, Node<K, V>>,
}

// Manual implementation as deriving `Clone` leads to unnecessary trait bounds.
// 手动实现，因为推导 `Clone` 会导致不必要的特征约束。
impl<K, V> Clone for Cursor<'_, K, V> {
    fn clone(&self) -> Self {
        Self {
            prev: self.prev,
            curr: self.curr,
        }
    }
}

impl<K, V> Node<K, V> {
    /// Creates a new node.
    /// 创建一个新节点。
    pub fn new(key: K, value: V) -> Self {
        Self {
            next: Atomic::null(),
            key,
            value,
        }
    }

    /// Extracts the inner value.
    /// 提取内部值。
    pub fn into_value(self) -> V {
        self.value
    }
}

impl<'g, K, V> Cursor<'g, K, V>
where
    K: Ord,
{
    /// Creates a cursor.
    /// 创建一个cursor。
    pub fn new(prev: &'g Atomic<Node<K, V>>, curr: Shared<'g, Node<K, V>>) -> Self {
        Self {
            prev,
            curr: curr.with_tag(0),
        }
    }

    /// Returns the current node.
    /// 返回当前节点。
    pub fn curr(&self) -> Shared<'g, Node<K, V>> {
        self.curr
    }

    /// Clean up a chain of logically removed nodes in each traversal.
    /// 在每次遍历中清理一串逻辑上被移除的节点。
    #[inline]
    pub fn find_harris(&mut self, key: &K, guard: &'g Guard) -> Result<bool, ()> {
        // Finding phase
        // 寻找阶段
        // - cursor.curr: first unmarked node w/ key >= search key (4)
        // - cursor.curr：第一个未标记且键值 >= 搜索键（4）的节点
        // - cursor.prev: the ref of .next in previous unmarked node (1 -> 2)
        // - cursor.prev：前一个未标记节点中 .next 的引用（1 -> 2）
        // 1 -> 2 -x-> 3 -x-> 4 -> 5 -> ∅  (search key: 4)
        // 1 -> 2 -x-> 3 -x-> 4 -> 5 -> ∅  （搜索键：4）
        let mut prev_next = self.curr;
        let found = loop {
            let Some(curr_node) = (unsafe { self.curr.as_ref() }) else {
                break false;
            };
            let next = curr_node.next.load(Acquire, guard);

            // - finding stage is done if cursor.curr advancement stops
            // - 如果 cursor.curr 进展停止，则查找阶段完成
            // - advance cursor.curr if (.next is marked) || (cursor.curr < key)
            // - 如果 (.next 被标记) 或 (cursor.curr < key)，则前进 cursor.curr
            // - stop cursor.curr if (not marked) && (cursor.curr >= key)
            // - 如果 (未标记) 且 (cursor.curr >= key) 则停止 cursor.curr
            // - advance cursor.prev if not marked
            // - 如果未标记，则前进 cursor.prev

            if next.tag() != 0 {
                // We add a 0 tag here so that `self.curr`s tag is always 0.
                // 我们在这里添加一个 0 标签，这样 `self.curr` 的标签总是 0。
                self.curr = next.with_tag(0);
                continue;
            }

            match curr_node.key.cmp(key) {
                Less => {
                    self.curr = next;
                    self.prev = &curr_node.next;
                    prev_next = next;
                }
                Equal => break true,
                Greater => break false,
            }
        };

        // If prev and curr WERE adjacent, no need to clean up
        // 如果 prev 和 curr 是相邻的，就不需要清理
        if prev_next == self.curr {
            return Ok(found);
        }

        // cleanup marked nodes between prev and curr
        // 清理 prev 和 curr 之间的标记节点
        self.prev
            .compare_exchange(prev_next, self.curr, Release, Relaxed, guard)
            .map_err(|_| ())?;

        // defer_destroy from cursor.prev.load() to cursor.curr (exclusive)
        // 从 cursor.prev.load() 到 cursor.curr（不包括）延迟销毁
        let mut node = prev_next;
        while node.with_tag(0) != self.curr {
            // SAFETY: All nodes in the unlinked chain are not null.
            // 安全性：未链接链中的所有节点都不为 null。
            //
            // NOTE: It may seem like this load could be non-atomic, but that would race with the
            // 注意：这个加载看起来可能是非原子的，但那会与……发生竞争
            // `fetch_or` done in `remove`.
            // `fetch_or` 在 `remove` 完成。
            let next = unsafe { node.deref() }.next.load(Relaxed, guard);

            // SAFETY: we unlinked the chain with above CAS.
            // 安全：我们用上述 CAS 解开了链条。
            unsafe { guard.defer_destroy(node) };
            node = next;
        }

        Ok(found)
    }

    /// Clean up a single logically removed node in each traversal.
    /// 在每次遍历中清理一个逻辑上已删除的节点。
    #[inline]
    pub fn find_harris_michael(&mut self, key: &K, guard: &'g Guard) -> Result<bool, ()> {
        loop {
            debug_assert_eq!(self.curr.tag(), 0);

            let Some(curr_node) = (unsafe { self.curr.as_ref() }) else {
                return Ok(false);
            };
            let mut next = curr_node.next.load(Acquire, guard);

            if next.tag() != 0 {
                next = next.with_tag(0);
                self.prev
                    .compare_exchange(self.curr, next, Release, Relaxed, guard)
                    .map_err(|_| ())?;
                unsafe { guard.defer_destroy(self.curr) };
                self.curr = next;
                continue;
            }

            match curr_node.key.cmp(key) {
                Less => {
                    self.prev = &curr_node.next;
                    self.curr = next;
                }
                Equal => return Ok(true),
                Greater => return Ok(false),
            }
        }
    }

    /// Doesn't preform any cleanup. Gotta go fast. Doesn't fail.
    /// 不进行任何清理。必须快速进行。不失败。
    #[inline]
    pub fn find_harris_herlihy_shavit(&mut self, key: &K, guard: &'g Guard) -> Result<bool, ()> {
        Ok(loop {
            let Some(curr_node) = (unsafe { self.curr.as_ref() }) else {
                break false;
            };
            match curr_node.key.cmp(key) {
                Less => {
                    // NOTE: unnecessary (this function is expected to be used only for `lookup`)
                    // 注意：不必要（此功能预计仅用于 `lookup`）
                    self.prev = &curr_node.next;
                    self.curr = curr_node.next.load(Acquire, guard);
                }
                Equal => break curr_node.next.load(Relaxed, guard).tag() == 0,
                Greater => break false,
            }
        })
    }

    /// Lookups the value at the current node.
    /// 查找当前节点的值。
    ///
    /// # Panics
    /// # panic
    ///
    /// Panics if the current node is null.
    /// 如果当前节点为空，则会引发panic。
    #[inline]
    pub fn lookup(&self) -> &'g V {
        &unsafe { self.curr.as_ref() }.unwrap().value
    }

    /// Inserts a value between the previous and current node.
    /// 在前一个节点和当前节点之间插入一个值。
    #[inline]
    pub fn insert(
        &mut self,
        mut node: Owned<Node<K, V>>,
        guard: &'g Guard,
    ) -> Result<(), Owned<Node<K, V>>> {
        node.next = self.curr.into();
        match self
            .prev
            .compare_exchange(self.curr, node, Release, Relaxed, guard)
        {
            Ok(node) => {
                self.curr = node;
                Ok(())
            }
            Err(e) => Err(e.new),
        }
    }

    /// Deletes the current node.
    /// 删除当前节点。
    ///
    /// # Panics
    /// # panic
    ///
    /// Panics if the current node is null.
    /// 如果当前节点为空，则会引发panic。
    #[inline]
    pub fn delete(&mut self, guard: &'g Guard) -> Result<&'g V, ()> {
        let curr_node = unsafe { self.curr.as_ref() }.unwrap();

        // Release: to release current view of the deleting thread on this mark.
        // 释放：在此标记上释放正在删除线程的当前视图。
        // Acquire: to ensure that if the latter CAS succeeds, then the thread that reads `next`
        // 获取：以确保如果后者 CAS 成功，那么读取 `next` 的线程
        // through `prev` will be safe.
        // 通过 `prev` 将是安全的。
        let next = curr_node.next.fetch_or(1, AcqRel, guard);
        if next.tag() == 1 {
            return Err(());
        }

        if self
            .prev
            .compare_exchange(self.curr, next, Release, Relaxed, guard)
            .is_ok()
        {
            // SAFETY: we are unlinker of curr. As the lifetime of the guard extends to the return
            // 安全性：我们是当前的断开器。随着保护器寿命延长至返回
            // value of the function, later access of curr_node is ok.
            // 函数的值，以后访问 curr_node 没问题。
            unsafe { guard.defer_destroy(self.curr) };
        }
        self.curr = next;

        Ok(&curr_node.value)
    }
}

impl<K, V> List<K, V>
where
    K: Ord,
{
    /// Creates a new list.
    /// 创建一个新的列表。
    pub fn new() -> Self {
        List {
            head: Atomic::null(),
        }
    }

    /// Creates the head cursor.
    /// 创建头部cursor。
    #[inline]
    pub fn head<'g>(&'g self, guard: &'g Guard) -> Cursor<'g, K, V> {
        Cursor::new(&self.head, self.head.load(Acquire, guard))
    }

    /// Finds a key using the given find strategy.
    /// 使用给定的查找策略查找键。
    #[inline]
    fn find<'g, F>(&'g self, key: &K, find: &F, guard: &'g Guard) -> (bool, Cursor<'g, K, V>)
    where
        F: Fn(&mut Cursor<'g, K, V>, &K, &'g Guard) -> Result<bool, ()>,
    {
        loop {
            let mut cursor = self.head(guard);
            if let Ok(r) = find(&mut cursor, key, guard) {
                return (r, cursor);
            }
        }
    }

    #[inline]
    fn lookup<'g, F>(&'g self, key: &K, find: F, guard: &'g Guard) -> Option<&'g V>
    where
        F: Fn(&mut Cursor<'g, K, V>, &K, &'g Guard) -> Result<bool, ()>,
    {
        let (found, cursor) = self.find(key, &find, guard);
        if found {
            // `found` means current node cannot be null, so lookup won't panic.
            // `found` 意味着当前节点不能为空，因此查找不会引发panic。
            Some(cursor.lookup())
        } else {
            None
        }
    }

    #[inline]
    fn insert<'g, F>(&'g self, key: K, value: V, find: F, guard: &'g Guard) -> bool
    where
        F: Fn(&mut Cursor<'g, K, V>, &K, &'g Guard) -> Result<bool, ()>,
    {
        let mut node = Owned::new(Node::new(key, value));
        loop {
            let (found, mut cursor) = self.find(&node.key, &find, guard);
            if found {
                return false;
            }

            match cursor.insert(node, guard) {
                Ok(()) => return true,
                Err(n) => node = n,
            }
        }
    }

    #[inline]
    fn delete<'g, F>(&'g self, key: &K, find: F, guard: &'g Guard) -> Option<&'g V>
    where
        F: Fn(&mut Cursor<'g, K, V>, &K, &'g Guard) -> Result<bool, ()>,
    {
        loop {
            let (found, mut cursor) = self.find(key, &find, guard);
            if !found {
                return None;
            }

            if let Ok(value) = cursor.delete(guard) {
                return Some(value);
            }
        }
    }

    /// Lookups the value at `key` with the Harris strategy.
    /// 使用 Harris 策略查找 `key` 的值。
    pub fn harris_lookup<'g>(&'g self, key: &K, guard: &'g Guard) -> Option<&'g V> {
        self.lookup(key, Cursor::find_harris, guard)
    }

    /// Insert the value with the Harris strategy.
    /// 使用哈里斯策略插入值。
    pub fn harris_insert<'g>(&'g self, key: K, value: V, guard: &'g Guard) -> bool {
        self.insert(key, value, Cursor::find_harris, guard)
    }

    /// Attempts to delete the value with the Harris strategy.
    /// 尝试使用哈里斯策略删除该值。
    pub fn harris_delete<'g>(&'g self, key: &K, guard: &'g Guard) -> Option<&'g V> {
        self.delete(key, Cursor::find_harris, guard)
    }

    /// Lookups the value at `key` with the Harris-Michael strategy.
    /// 使用 Harris-Michael 策略查找 `key` 的值。
    pub fn harris_michael_lookup<'g>(&'g self, key: &K, guard: &'g Guard) -> Option<&'g V> {
        self.lookup(key, Cursor::find_harris_michael, guard)
    }

    /// Insert a `key`-`value`` pair with the Harris-Michael strategy.
    /// 使用 Harris-Michael 策略插入一个 `key`-`value`` 对。
    pub fn harris_michael_insert(&self, key: K, value: V, guard: &Guard) -> bool {
        self.insert(key, value, Cursor::find_harris_michael, guard)
    }

    /// Delete the value at `key` with the Harris-Michael strategy.
    /// 使用 Harris-Michael 策略删除 `key` 的值。
    pub fn harris_michael_delete<'g>(&'g self, key: &K, guard: &'g Guard) -> Option<&'g V> {
        self.delete(key, Cursor::find_harris_michael, guard)
    }

    /// Lookups the value at `key` with the Harris-Herlihy-Shavit strategy.
    /// 使用 Harris-Herlihy-Shavit 策略查找 `key` 处的值。
    pub fn harris_herlihy_shavit_lookup<'g>(&'g self, key: &K, guard: &'g Guard) -> Option<&'g V> {
        self.lookup(key, Cursor::find_harris_herlihy_shavit, guard)
    }
}
