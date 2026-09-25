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
    /// Invariant upheld here: on *every* return path (`Ok` or `Err`), `self.prev` is left as a
    /// live, un-`finish`ed `ReadGuard`. Because `ReadGuard::drop` panics, the caller is responsible
    /// for eventually `finish`ing / `upgrade`ing it (even on the `Err` path).
    /// 本函数维持的不变式：在*每一条*返回路径（`Ok` 或 `Err`）上，`self.prev` 都保持为一个
    /// 存活、未 `finish` 的 `ReadGuard`。由于 `ReadGuard::drop` 会 panic，调用方负责最终
    /// 对它 `finish` / `upgrade`（即使在 `Err` 路径上也是如此）。
    fn find(&mut self, key: &T, guard: &'g Guard) -> Result<bool, ()> {
        loop {
            // `self.curr` is what `self.prev` pointed to when we read it. Before we are allowed to
            // dereference it, we must be sure that snapshot was consistent, i.e. no writer changed
            // `prev.next` while we were reading it.
            // `self.curr` 是我们读取 `self.prev` 时它所指向的值。在允许解引用它之前，
            // 我们必须确保那份快照是一致的，即在我们读取 `prev.next` 期间没有写者修改过它。
            let Some(node) = (unsafe { self.curr.as_ref() }) else {
                // Reached the end of the list. Validate the read of the tail's `next` (null) so the
                // caller knows the cursor position is trustworthy.
                // 到达列表末尾。验证对末尾 `next`（null）的读取，让调用方知道 cursor 位置可信。
                //
                // We only `validate()` (borrows `&self`) rather than `finish()` (consumes `self`),
                // keeping the `self.prev` invariant above.
                // 这里只用 `validate()`（借用 `&self`）而非 `finish()`（消费 `self`），
                // 以维持上面关于 `self.prev` 的不变式。
                return if self.prev.validate() {
                    Ok(false)
                } else {
                    Err(())
                };
            };

            match node.data.cmp(key) {
                Less => {
                    // We want to move past `node`, so we start a read section on `node.next`.
                    // 我们想越过 `node`，所以在 `node.next` 上开启一段读临界区。
                    let next = unsafe { node.next.read_lock() };
                    let next_shared = next.load(SeqCst, guard);

                    // Advance the cursor: swap `next` into `self.prev` and take the old guard out.
                    // `mem::replace` avoids two illegal moves: (1) moving `self.prev` out from
                    // behind `&mut self`, and (2) an assignment `self.prev = next` that would *drop*
                    // the old guard and thus panic.
                    // 推进 cursor：把 `next` 换入 `self.prev`，并取出旧的 guard。`mem::replace`
                    // 规避了两种非法 move：(1) 从 `&mut self` 后面把 `self.prev` move 出去；
                    // (2) 用 `self.prev = next` 赋值会 *drop* 旧 guard 从而 panic。
                    let prev = mem::replace(&mut self.prev, next);
                    self.curr = next_shared;

                    // Hand-over-hand *validation*: confirm the snapshot that led us to `node` was
                    // valid. `finish()` validates and forgets `prev` (so it won't panic-drop). If
                    // it fails, `self.prev` is still the live `next` guard, honoring the invariant.
                    // 逐步传递式的*验证*：确认引导我们到达 `node` 的那份快照有效。`finish()` 会
                    // 验证并 forget `prev`（因此不会 panic-drop）。若失败，`self.prev` 仍是存活的
                    // `next` guard，满足不变式。
                    if !prev.finish() {
                        return Err(());
                    }
                }
                Equal => {
                    return if self.prev.validate() {
                        Ok(true)
                    } else {
                        Err(())
                    };
                }
                Greater => {
                    return if self.prev.validate() {
                        Ok(false)
                    } else {
                        Err(())
                    };
                }
            }
        }
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
        // `head` 中保存的是原子指针；epoch guard 保证加载到的节点在本次访问期间不会被释放。
        let curr = prev.load(SeqCst, guard);
        Cursor { prev, curr }
    }
}

impl<T: Ord> OptimisticFineGrainedListSet<T> {
    fn find<'g>(&'g self, key: &T, guard: &'g Guard) -> Result<(bool, Cursor<'g, T>), ()> {
        let mut cursor = self.head(guard);

        match cursor.find(key, guard) {
            Ok(found) => Ok((found, cursor)),
            Err(()) => {
                // `ReadGuard` 不能直接析构。即使前一次校验已经失败，也必须用 `finish`
                // 正常结束当前读临界区；返回值在这里无需再次处理。
                let _ = cursor.prev.finish();
                Err(())
            }
        }
    }
}

impl<T: Ord> ConcurrentSet<T> for OptimisticFineGrainedListSet<T> {
    fn contains(&self, key: &T) -> bool {
        let guard = pin();

        loop {
            let Ok((found, cursor)) = self.find(key, &guard) else {
                // 遍历期间发生了写入，本次快照无效，从链表头重新尝试。
                continue;
            };

            // `find` 返回到这里后，写线程仍可能修改 `prev`。只有最终校验成功，
            // 才能把 `found` 当作一次有效的查询结果。
            if cursor.prev.finish() {
                return found;
            }
        }
    }

    fn insert(&self, key: T) -> bool {
        let guard = pin();

        loop {
            let Ok((found, cursor)) = self.find(&key, &guard) else {
                continue;
            };

            if found {
                // 集合中已有相同元素。最终校验成功才能确认它确实存在。
                if cursor.prev.finish() {
                    return false;
                }
                continue;
            }

            // `prev` 正是指向插入位置的指针字段。升级失败表示它已被别的写线程
            // 修改，此时必须丢弃本次定位结果并重试。
            let Ok(prev) = cursor.prev.upgrade() else {
                continue;
            };

            // 新节点接在 `prev` 与原来的 `curr` 之间。写锁释放时序列号会变化，
            // 所有读到旧链路的乐观读者都会在校验时发现冲突。
            prev.store(Node::new(key, cursor.curr), SeqCst);
            return true;
        }
    }

    fn remove(&self, key: &T) -> bool {
        let guard = pin();

        loop {
            let Ok((found, cursor)) = self.find(key, &guard) else {
                continue;
            };

            if !found {
                // 最终校验成功后，才能确认目标确实不在集合中。
                if cursor.prev.finish() {
                    return false;
                }
                continue;
            }

            let Ok(prev) = cursor.prev.upgrade() else {
                continue;
            };

            // `find` 已确认 `curr` 非空且由 `prev` 指向；升级成功又保证这条边没有
            // 被其他写线程改动，因此这里可以安全访问当前节点。
            let curr = unsafe { cursor.curr.deref() };

            // 删除节点时还要锁住它的 `next`。这样在读取后继并改写 `prev` 的过程中，
            // 不会有另一个线程在当前节点之后插入或删除。
            let next = curr.next.write_lock();
            let successor = next.load(SeqCst, &guard);
            prev.store(successor, SeqCst);
            drop(next);

            // 节点已经从链表摘除，但其他乐观读者可能仍持有它的地址。
            // epoch 回收会等到这些读者离开各自的临界区后再真正释放节点。
            unsafe { guard.defer_destroy(cursor.curr) };
            return true;
        }
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
        let cursor = &mut *self.cursor;

        let Some(node) = (unsafe { cursor.curr.as_ref() }) else {
            // 即使已经走到末尾，也要验证“这里是 null”这一观察是否仍然有效。
            return if cursor.prev.validate() {
                None
            } else {
                Some(Err(()))
            };
        };

        // 先为下一条边建立读快照，再结束上一条边的读快照。这与 `Cursor::find`
        // 的逐步传递方式相同，并且不会阻塞写线程。
        let next = unsafe { node.next.read_lock() };
        let next_shared = next.load(SeqCst, self.guard);
        let prev = mem::replace(&mut cursor.prev, next);
        cursor.curr = next_shared;

        if prev.finish() {
            // `node` 的内存由 `self.guard` 保护；节点的数据一经创建便不再修改。
            Some(Ok(&node.data))
        } else {
            // 调用者看到错误后应丢弃迭代器，并从头创建新的迭代器。
            Some(Err(()))
        }
    }
}

impl<T> Drop for OptimisticFineGrainedListSet<T> {
    fn drop(&mut self) {
        // `&mut self` 保证此时没有线程还能访问链表，因此无需加锁或延迟回收。
        // 取出头指针后逐个把 `Atomic` 恢复成独占的 `Owned`，离开循环时节点会析构。
        let mut curr = mem::take(&mut self.head).into_inner();
        while let Some(node) = unsafe { curr.try_into_owned() }.map(Owned::into_box) {
            curr = node.next.into_inner();
        }
    }
}

impl<T> Default for OptimisticFineGrainedListSet<T> {
    fn default() -> Self {
        Self::new()
    }
}
