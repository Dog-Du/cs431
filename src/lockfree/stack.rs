use core::mem::{self, MaybeUninit};
use core::ptr;
use core::sync::atomic::Ordering::*;

use crossbeam_epoch::{Atomic, Owned, Shared};

/// Treiber's lock-free stack.
/// Treiber 的无锁栈。
///
/// Usable with any number of producers and consumers.
/// 可用于任意数量的生产者和消费者。
#[derive(Debug)]
pub struct Stack<T> {
    head: Atomic<Node<T>>,
}

#[derive(Debug)]
struct Node<T> {
    // MaybeUninit as the data may be taken out of the node.
    // MaybeUninit，因为数据可能会被取出节点。
    // TODO: fix the slides to sync with this.
    // 待办：修正幻灯片以与此同步。
    data: MaybeUninit<T>,
    next: *const Node<T>,
}

// Any particular `T` should never be accessed concurrently, so no need for `Sync`.
// 任何特定的 `T` 都不应同时访问，因此不需要 `Sync`。
unsafe impl<T: Send> Send for Stack<T> {}
unsafe impl<T: Send> Sync for Stack<T> {}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Stack<T> {
    /// Creates a new, empty stack.
    /// 创建一个新的空栈。
    pub fn new() -> Stack<T> {
        Self {
            head: Atomic::null(),
        }
    }

    /// Pushes a value on top of the stack.
    /// 将一个值推入栈顶。
    pub fn push(&self, t: T) {
        let mut node = Owned::new(Node {
            data: MaybeUninit::new(t),
            next: ptr::null(),
        });

        // SAFETY: We don't dereference any pointers obtained from this guard.
        // 安全性：我们不会解引用从此保护获取的任何指针。
        let guard = unsafe { crossbeam_epoch::unprotected() };

        let mut head = self.head.load(Relaxed, guard);
        loop {
            node.next = head.as_raw();

            match self
                .head
                .compare_exchange(head, node, Release, Relaxed, guard)
            {
                Ok(_) => break,
                Err(e) => {
                    head = e.current;
                    node = e.new;
                }
            }
        }
    }

    /// Attempts to pop the top element from the stack.
    /// 尝试从栈中弹出顶部元素。
    ///
    /// Returns `None` if the stack is empty.
    /// 如果栈为空，则返回 `None`。
    pub fn pop(&self) -> Option<T> {
        let mut guard = crossbeam_epoch::pin();

        loop {
            let head = self.head.load(Acquire, &guard);
            let h = unsafe { head.as_ref() }?;
            let next = Shared::from(h.next);

            if self
                .head
                .compare_exchange(head, next, Relaxed, Relaxed, &guard)
                .is_ok()
            {
                // Since the above `compare_exchange()` succeeded, `head` is detached from
                // 由于上述 `compare_exchange()` 成功，`head` 与…分离
                // `self` so is unreachable from other threads.
                // `self` 因此无法从其他线程访问。

                // SAFETY: We are returning ownership of `data` in `head` by making a copy of it via
                // 安全：我们正在通过复制它来归还 `head` 中 `data` 的所有权
                // `assume_init_read()`. This is safe as no other thread has access to `data` after
                // `assume_init_read()`。这是安全的，因为在此之后没有其他线程可以访问 `data`
                // `head` is unreachable, so the ownership of `data` in `head` will never be used
                // `head` 无法访问，因此 `head` 中 `data` 的所有权将永远不会被使用
                // again.
                // 再次。
                let result = unsafe { h.data.assume_init_read() };

                // SAFETY: `head` is unreachable, and we no longer access `head`.
                // 安全：`head`无法访问，我们也不再访问`head`。
                unsafe { guard.defer_destroy(head) };

                return Some(result);
            }

            // Repin to ensure the global epoch can make progress.
            // 重新钉住以确保全球时代能够进步。
            guard.repin();
        }
    }

    /// Returns `true` if the stack is empty.
    /// 如果堆栈为空，则返回 `true`。
    pub fn is_empty(&self) -> bool {
        let guard = crossbeam_epoch::pin();
        self.head.load(Acquire, &guard).is_null()
    }
}

impl<T> Drop for Stack<T> {
    fn drop(&mut self) {
        let mut o_curr = mem::take(&mut self.head);

        // SAFETY: All non-null nodes made were valid, and we have unique ownership via `&mut self`.
        // 安全性：所有创建的非空节点都是有效的，并且我们通过 `&mut self` 拥有唯一所有权。
        while let Some(curr) = unsafe { o_curr.try_into_owned() }.map(Owned::into_box) {
            drop(unsafe { curr.data.assume_init() });
            o_curr = curr.next.into();
        }
    }
}

#[cfg(test)]
mod test {
    use std::thread::scope;

    use super::*;

    #[test]
    fn push() {
        let stack = Stack::new();

        scope(|scope| {
            for _ in 0..10 {
                scope.spawn(|| {
                    for i in 0..10_000 {
                        stack.push(i);
                        assert!(stack.pop().is_some());
                    }
                });
            }
        });

        assert!(stack.is_empty());
    }
}
