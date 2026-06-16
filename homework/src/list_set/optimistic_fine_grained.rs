use std::cmp::Ordering::*;
use std::mem::{self, ManuallyDrop};
use std::sync::atomic::Ordering::*;

use crossbeam_epoch::{Atomic, Guard, Owned, Shared, pin};
use cs431::lock::seqlock::{ReadGuard, SeqLock};

use crate::ConcurrentSet;

#[derive(Debug)]
struct Node<T> {
    data: T,
    next: SeqLock<Atomic<Node<T>>>,
}

/// Concurrent sorted singly linked list using fine-grained optimistic locking.
/// 使用细粒度乐观锁的并发有序单向链表。
#[derive(Debug)]
pub struct OptimisticFineGrainedListSet<T> {
    head: SeqLock<Atomic<Node<T>>>,
}

unsafe impl<T: Send> Send for OptimisticFineGrainedListSet<T> {}
unsafe impl<T: Sync> Sync for OptimisticFineGrainedListSet<T> {}

#[derive(Debug)]
struct Cursor<'g, T> {
    // Reference to the `next` field of previous node which points to the current node.
    // 引用前一个节点的 `next` 字段，该字段指向当前节点。
    prev: ReadGuard<'g, Atomic<Node<T>>>,
    curr: Shared<'g, Node<T>>,
}

impl<T> Node<T> {
    fn new(data: T, next: Shared<'_, Self>) -> Owned<Self> {
        Owned::new(Self {
            data,
            next: SeqLock::new(next.into()),
        })
    }
}

impl<'g, T: Ord> Cursor<'g, T> {
    /// Moves the cursor to the position of key in the sorted list.
    /// 将cursor移动到排序列表中键的位置。
    /// Returns whether the value was found.
    /// 返回是否找到该值。
    ///
    /// Return `Err(())` if the cursor cannot move.
    /// 如果cursor无法移动，请返回 `Err(())`。
    fn find(&mut self, key: &T, guard: &'g Guard) -> Result<bool, ()> {
        todo!()
    }
}

impl<T> OptimisticFineGrainedListSet<T> {
    /// Creates a new list.
    /// 创建一个新的列表。
    pub fn new() -> Self {
        Self {
            head: SeqLock::new(Atomic::null()),
        }
    }

    fn head<'g>(&'g self, guard: &'g Guard) -> Cursor<'g, T> {
        let prev = unsafe { self.head.read_lock() };
        let curr = prev.load(Acquire, guard);
        Cursor { prev, curr }
    }
}

impl<T: Ord> OptimisticFineGrainedListSet<T> {
    fn find<'g>(&'g self, key: &T, guard: &'g Guard) -> Result<(bool, Cursor<'g, T>), ()> {
        todo!()
    }
}

impl<T: Ord> ConcurrentSet<T> for OptimisticFineGrainedListSet<T> {
    fn contains(&self, key: &T) -> bool {
        todo!()
    }

    fn insert(&self, key: T) -> bool {
        todo!()
    }

    fn remove(&self, key: &T) -> bool {
        todo!()
    }
}

#[derive(Debug)]
pub struct Iter<'g, T> {
    // Can be dropped without validation, because the only way to use cursor.curr is next().
    // 可以在不进行验证的情况下丢弃，因为使用 cursor.curr 的唯一方法是 next()。
    cursor: ManuallyDrop<Cursor<'g, T>>,
    guard: &'g Guard,
}

impl<T> OptimisticFineGrainedListSet<T> {
    /// An iterator visiting all elements. `next()` returns `Some(Err(()))` when validation fails.
    /// 一个遍历所有元素的迭代器。当验证失败时，`next()` 返回 `Some(Err(()))`。
    /// In that case, the user must restart the iteration.
    /// 在那种情况下，用户必须重新启动迭代。
    pub fn iter<'g>(&'g self, guard: &'g Guard) -> Iter<'g, T> {
        Iter {
            cursor: ManuallyDrop::new(self.head(guard)),
            guard,
        }
    }
}

impl<'g, T> Iterator for Iter<'g, T> {
    type Item = Result<&'g T, ()>;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<T> Drop for OptimisticFineGrainedListSet<T> {
    fn drop(&mut self) {
        todo!()
    }
}

impl<T> Default for OptimisticFineGrainedListSet<T> {
    fn default() -> Self {
        Self::new()
    }
}
