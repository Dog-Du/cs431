//! Michael-Scott lock-free queue.
//! Michael-Scott 无锁队列。
//!
//! Usable with any number of producers and consumers.
//! 可用于任意数量的生产者和消费者。
//!
//! Michael and Scott.  Simple, Fast, and Practical Non-Blocking and Blocking Concurrent Queue
//! Michael 和 Scott。简单、快速且实用的非阻塞和阻塞并发队列
//! Algorithms.  PODC 1996.  <http://dl.acm.org/citation.cfm?id=248106>
//! 算法。PODC 1996。<http://dl.acm.org/citation.cfm?id=248106>

use core::mem::{self, MaybeUninit};
use core::sync::atomic::Ordering::*;

use crossbeam_epoch::{Atomic, Guard, Owned, Shared};
use crossbeam_utils::CachePadded;

/// Michael-Scott queue.
/// 迈克尔-斯科特队列。
// The representation here is a singly-linked list, with a sentinel node at the front. In general
// 这里的表示是一个单链表，前面有一个哨兵节点。一般来说
// the `tail` pointer may lag behind the actual tail.
// `tail` 指针可能会落后于实际尾部。
#[derive(Debug)]
pub struct Queue<T> {
    head: CachePadded<Atomic<Node<T>>>,
    tail: CachePadded<Atomic<Node<T>>>,
}

#[derive(Debug)]
struct Node<T> {
    /// The place in which a value of type `T` can be stored.
    /// 可以存储类型为 `T` 的值的位置。
    ///
    /// The type of `data` is `MaybeUninit<T>` because a `Node<T>` doesn't always contain a `T`.
    /// `data` 的类型是 `MaybeUninit<T>`，因为 `Node<T>` 并不总是包含 `T`。
    /// For example, the initial sentinel node in a queue never contains a value: its data is
    /// 例如，队列中的初始哨兵节点从不包含值：它的数据是
    /// always uninitialized. Other nodes start their life with a push operation and contain a
    /// 总是未初始化。其他节点的生命周期以一次推入操作开始，并包含一个
    /// value until it gets popped out.
    /// 值直到它被弹出为止。
    data: MaybeUninit<T>,

    next: Atomic<Node<T>>,
}

// Any particular `T` should never be accessed concurrently, so no need for `Sync`.
// 任何特定的 `T` 都不应同时访问，因此不需要 `Sync`。
unsafe impl<T: Send> Sync for Queue<T> {}
unsafe impl<T: Send> Send for Queue<T> {}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Queue<T> {
    /// Create a new, empty queue.
    /// 创建一个新的空队列。
    pub fn new() -> Self {
        let sentinel = Box::into_raw(Box::new(Node {
            data: MaybeUninit::uninit(),
            next: Atomic::null(),
        }))
        .cast_const();

        Self {
            head: CachePadded::new(sentinel.into()),
            tail: CachePadded::new(sentinel.into()),
        }
    }

    /// Adds `t` to the back of the queue.
    /// 将 `t` 添加到队列的末尾。
    pub fn push(&self, t: T, guard: &mut Guard) {
        let mut new = Owned::new(Node {
            data: MaybeUninit::new(t),
            next: Atomic::null(),
        });

        loop {
            // We push onto the tail, so we'll start optimistically by looking there first.
            // 我们从尾部开始推进，所以我们会以乐观的态度先从那里开始查看。
            let tail = self.tail.load(Acquire, guard);

            // Attempt to push onto the `tail` snapshot; fails if `tail.next` has changed.
            // 尝试推送到 `tail` 快照；如果 `tail.next` 已更改则失败。
            let tail_ref = unsafe { tail.deref() };
            let next = tail_ref.next.load(Acquire, guard);

            // If `tail` is not the actual tail, try to "help" by moving the tail pointer forward.
            // 如果 `tail` 不是实际的尾部，尝试通过将尾指针向前移动来“帮助”。
            if !next.is_null() {
                let _ = self
                    .tail
                    .compare_exchange(tail, next, Release, Relaxed, guard);
                continue;
            }

            // looks like the actual tail; attempt to link at `tail.next`.
            // 看起来像真正的尾巴；尝试在 `tail.next` 链接。
            match tail_ref
                .next
                .compare_exchange(Shared::null(), new, Release, Relaxed, guard)
            {
                Ok(new) => {
                    // try to move the tail pointer forward.
                    // 尝试将尾指针向前移动。
                    let _ = self
                        .tail
                        .compare_exchange(tail, new, Release, Relaxed, guard);
                    break;
                }
                Err(e) => new = e.new,
            }
            guard.repin();
        }
    }

    /// Attempts to dequeue from the front.
    /// 尝试从队列前端出队。
    ///
    /// Returns `None` if the queue is observed to be empty.
    /// 如果观察到队列为空，则返回 `None`。
    pub fn try_pop(&self, guard: &mut Guard) -> Option<T> {
        loop {
            let head = self.head.load(Acquire, guard);
            let next = unsafe { head.deref() }.next.load(Acquire, guard);

            let next_ref = unsafe { next.as_ref() }?;

            // Moves `tail` if it's stale. Relaxed load is enough because if tail == head, then the
            // 如果 `tail` 过期，则移动它。使用放松加载就够了，因为如果 tail == head，那么
            // messages for that node are already acquired.
            // 该节点的消息已被获取。
            let tail = self.tail.load(Relaxed, guard);
            if tail == head {
                let _ = self
                    .tail
                    .compare_exchange(tail, next, Release, Relaxed, guard);
            }

            // After the above load & CAS, the thread view ensures that the index of tail is greater
            // 在上述加载和 CAS 之后，线程视图确保尾部的索引更大
            // than that of current head. We relase that view to the head with the below CAS,
            // 比当前头部的要大。我们将使用下面的CAS将该视图释放给头部，
            // ensuring that the index of the new head is less than or equal to that of the tail.
            // 确保新头的索引小于或等于尾部的索引。
            //
            // Note: this reasoning is also done in SC memory regarding index of head and tail,
            // 注意：这种推理在 SC 内存中关于头和尾的索引也同样适用，
            // albeit simpler.
            // 尽管更简单。
            if self
                .head
                .compare_exchange(head, next, Release, Relaxed, guard)
                .is_ok()
            {
                // Since the above `compare_exchange()` succeeded, `head` is detached from `self` so
                // 由于上述 `compare_exchange()` 成功，`head` 从 `self` 分离，因此
                // is unreachable from other threads.
                // 无法从其他线程访问。

                // SAFETY: `next` will never be the sentinel node, since it is the node after
                // 安全性：`next` 永远不会是哨兵节点，因为它是之后的节点
                // `head`. Hence, it must have been a node made in `push()`, which is initialized.
                // `head`。因此，它一定是在已初始化的 `push()` 中创建的节点。
                //
                // Also, we are returning ownership of `data` in `next` by making a copy of it via
                // 此外，我们正在通过复制它来返还 `next` 中 `data` 的所有权
                // `assume_init_read()`. This is safe as no other thread has access to `data` after
                // `assume_init_read()`。这是安全的，因为在此之后没有其他线程可以访问 `data`
                // `head` is unreachable, so the ownership of `data` in `next` will never be used
                // `head` 无法访问，因此 `next` 中 `data` 的所有权将永远不会被使用
                // again as it is now a sentinel node.
                // 又一次，因为它现在是一个哨兵节点。
                let result = unsafe { next_ref.data.assume_init_read() };

                // SAFETY: `head` is unreachable, and we no longer access `head`. We destroy `head`
                // 安全：`head`无法到达，我们不再访问`head`。我们销毁`head`
                // after the final access to `next` above to ensure that `next` is also destroyed
                // 在上述对 `next` 最后的访问之后，以确保 `next` 也被销毁
                // after.
                // 之后。
                unsafe { guard.defer_destroy(head) };

                return Some(result);
            }
            guard.repin();
        }
    }
}

impl<T> Drop for Queue<T> {
    fn drop(&mut self) {
        // Destroy the sentinel node.
        // 摧毁哨兵节点。

        let sentinel = mem::take(&mut *self.head);
        // SAFETY: `pop()` never dropped the sentinel node so it is still valid.
        // 安全性：`pop()` 从未丢弃哨兵节点，所以它仍然有效。
        let mut o_curr = unsafe { sentinel.into_owned() }.into_box().next;

        // Destroy and deallocate `data` for the rest of the nodes.
        // 销毁并释放其余节点的 `data`。

        // SAFETY: All non-null nodes made were valid, and we have unique ownership via `&mut self`.
        // 安全性：所有创建的非空节点都是有效的，并且我们通过 `&mut self` 拥有唯一所有权。
        while let Some(curr) = unsafe { o_curr.try_into_owned() }.map(Owned::into_box) {
            // SAFETY: Not sentinel node, so `data` is valid.
            // 安全性：不是哨兵节点，所以 `data` 是有效的。
            drop(unsafe { curr.data.assume_init() });
            o_curr = curr.next;
        }
    }
}

#[cfg(test)]
mod test {
    use std::thread::scope;

    use crossbeam_epoch::pin;

    use super::*;

    struct Queue<T> {
        queue: super::Queue<T>,
    }

    impl<T> Queue<T> {
        pub fn new() -> Queue<T> {
            Queue {
                queue: super::Queue::new(),
            }
        }

        pub fn push(&self, t: T) {
            let guard = &mut pin();
            self.queue.push(t, guard);
        }

        pub fn is_empty(&self) -> bool {
            let guard = &pin();
            let head = self.queue.head.load(Acquire, guard);
            let next = unsafe { head.deref() }.next.load(Acquire, guard);
            next.is_null()
        }

        pub fn try_pop(&self) -> Option<T> {
            let guard = &mut pin();
            self.queue.try_pop(guard)
        }

        pub fn pop(&self) -> T {
            loop {
                if let Some(t) = self.try_pop() {
                    return t;
                }
            }
        }
    }

    const CONC_COUNT: i64 = 1000000;

    #[test]
    fn push_try_pop_1() {
        let q: Queue<i64> = Queue::new();
        assert!(q.is_empty());
        q.push(37);
        assert!(!q.is_empty());
        assert_eq!(q.try_pop(), Some(37));
        assert!(q.is_empty());
    }

    #[test]
    fn push_try_pop_2() {
        let q: Queue<i64> = Queue::new();
        assert!(q.is_empty());
        q.push(37);
        q.push(48);
        assert_eq!(q.try_pop(), Some(37));
        assert!(!q.is_empty());
        assert_eq!(q.try_pop(), Some(48));
        assert!(q.is_empty());
    }

    #[test]
    fn push_try_pop_many_seq() {
        let q: Queue<i64> = Queue::new();
        assert!(q.is_empty());
        for i in 0..200 {
            q.push(i)
        }
        assert!(!q.is_empty());
        for i in 0..200 {
            assert_eq!(q.try_pop(), Some(i));
        }
        assert!(q.is_empty());
    }

    #[test]
    fn push_pop_1() {
        let q: Queue<i64> = Queue::new();
        assert!(q.is_empty());
        q.push(37);
        assert!(!q.is_empty());
        assert_eq!(q.pop(), 37);
        assert!(q.is_empty());
    }

    #[test]
    fn push_pop_2() {
        let q: Queue<i64> = Queue::new();
        q.push(37);
        q.push(48);
        assert_eq!(q.pop(), 37);
        assert_eq!(q.pop(), 48);
    }

    #[test]
    fn push_pop_many_seq() {
        let q: Queue<i64> = Queue::new();
        assert!(q.is_empty());
        for i in 0..200 {
            q.push(i)
        }
        assert!(!q.is_empty());
        for i in 0..200 {
            assert_eq!(q.pop(), i);
        }
        assert!(q.is_empty());
    }

    #[test]
    fn push_try_pop_many_spsc() {
        let q: Queue<i64> = Queue::new();
        assert!(q.is_empty());

        scope(|scope| {
            scope.spawn(|| {
                let mut next = 0;

                while next < CONC_COUNT {
                    if let Some(elem) = q.try_pop() {
                        assert_eq!(elem, next);
                        next += 1;
                    }
                }
            });

            for i in 0..CONC_COUNT {
                q.push(i)
            }
        });
    }

    #[test]
    fn push_try_pop_many_spmc() {
        fn recv(q: &Queue<i64>) {
            let mut cur = -1;
            for _ in 0..CONC_COUNT {
                if let Some(elem) = q.try_pop() {
                    assert!(elem > cur);
                    cur = elem;

                    if cur == CONC_COUNT - 1 {
                        break;
                    }
                }
            }
        }

        let q: Queue<i64> = Queue::new();
        assert!(q.is_empty());
        scope(|scope| {
            for _ in 0..3 {
                scope.spawn(|| recv(&q));
            }

            scope.spawn(|| {
                for i in 0..CONC_COUNT {
                    q.push(i);
                }
            });
        });
    }

    #[test]
    fn push_try_pop_many_mpmc() {
        enum LR {
            Left(i64),
            Right(i64),
        }

        let q: Queue<LR> = Queue::new();
        assert!(q.is_empty());

        scope(|scope| {
            scope.spawn(|| {
                for i in 0..CONC_COUNT {
                    q.push(LR::Left(i))
                }
            });
            scope.spawn(|| {
                for i in 0..CONC_COUNT {
                    q.push(LR::Right(i))
                }
            });
            for _ in 0..2 {
                scope.spawn(|| {
                    let mut vl = vec![];
                    let mut vr = vec![];
                    for _ in 0..CONC_COUNT {
                        match q.try_pop() {
                            Some(LR::Left(x)) => vl.push(x),
                            Some(LR::Right(x)) => vr.push(x),
                            _ => {}
                        }
                    }

                    let mut vl2 = vl.clone();
                    let mut vr2 = vr.clone();
                    vl2.sort();
                    vr2.sort();

                    assert_eq!(vl, vl2);
                    assert_eq!(vr, vr2);
                });
            }
        });
    }

    #[test]
    fn push_pop_many_spsc() {
        let q: Queue<i64> = Queue::new();

        scope(|scope| {
            scope.spawn(|| {
                let mut next = 0;
                while next < CONC_COUNT {
                    assert_eq!(q.pop(), next);
                    next += 1;
                }
            });

            for i in 0..CONC_COUNT {
                q.push(i)
            }
        });
        assert!(q.is_empty());
    }

    #[test]
    fn is_empty_dont_pop() {
        let q: Queue<i64> = Queue::new();
        q.push(20);
        q.push(20);
        assert!(!q.is_empty());
        assert!(q.try_pop().is_some());
    }
}
