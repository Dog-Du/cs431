use core::ptr;
use core::sync::atomic::Ordering::*;
use core::sync::atomic::{AtomicBool, AtomicPtr};

use crossbeam_utils::{Backoff, CachePadded};

use crate::lock::*;

struct Node {
    locked: AtomicBool,
    next: AtomicPtr<CachePadded<Node>>,
}

#[derive(Debug, Clone)]
pub struct Token(*mut CachePadded<Node>);

// SAFETY: It doesn't matter if a thread used a token made by another thread.
// 安全性：线程使用另一个线程生成的令牌没有关系。
unsafe impl Send for Token {}

/// An MCS lock.
/// MCS锁。
#[derive(Debug)]
pub struct McsLock {
    tail: AtomicPtr<CachePadded<Node>>,
}

impl Node {
    fn new() -> *mut CachePadded<Self> {
        Box::into_raw(Box::new(CachePadded::new(Self {
            locked: AtomicBool::new(true),
            next: AtomicPtr::new(ptr::null_mut()),
        })))
    }
}

impl Default for McsLock {
    fn default() -> Self {
        Self {
            tail: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

unsafe impl RawLock for McsLock {
    type Token = Token;

    fn lock(&self) -> Self::Token {
        let node = Node::new();
        let prev = self.tail.swap(node, AcqRel);

        if prev.is_null() {
            return Token(node);
        }

        // SAFETY: `prev` is valid, so is not the initial pointer. Hence, it is a pointer from
        // 安全性：`prev` 是有效的，但不是初始指针。因此，它是一个指向
        // `swap()` by another thread's `lock()`, and that thread guarantees that `prev` will not be
        // `swap()` 通过另一个线程的 `lock()`，并且该线程保证 `prev` 不会发生
        // freed until this store is complete.
        // 直到这家商店完成之前自由。
        unsafe { (*prev).next.store(node, Release) };

        let backoff = Backoff::new();
        // SAFETY: `node` was made valid above. Since other threads will not free `node`, it still
        // 安全性：`node` 已在上文被设为有效。由于其他线程不会释放 `node`，它仍然
        // points to valid memory.
        // 指向有效内存。
        while unsafe { (*node).locked.load(Acquire) } {
            backoff.snooze();
        }

        Token(node)
    }

    unsafe fn unlock(&self, token: Self::Token) {
        let node = token.0;
        let mut next = unsafe { (*node).next.load(Acquire) };

        if next.is_null() {
            if self
                .tail
                .compare_exchange(node, ptr::null_mut(), Release, Relaxed)
                .is_ok()
            {
                // SAFETY: Since `node` was the `tail`, there is no other thread blocked by this
                // 安全性：由于 `node` 是 `tail`，因此没有其他线程被此阻塞
                // lock. Hence we have unique access to it.
                // 锁。因此我们对它有唯一的访问权限。
                drop(unsafe { Box::from_raw(node) });
                return;
            }

            while {
                next = unsafe { (*node).next.load(Acquire) };
                next.is_null()
            } {}
        }

        // SAFETY: Since `next` is not null, the thread that made `next` has finished access to
        // 安全性：由于 `next` 非空，创建 `next` 的线程已经完成访问
        // `node`, hence we have unique access to it.
        // `node`，因此我们拥有对它的独特访问权限。
        drop(unsafe { Box::from_raw(node) });
        unsafe { (*next).locked.store(false, Release) };
    }
}

#[cfg(test)]
mod tests {
    use super::super::api;
    use super::McsLock;

    #[test]
    fn smoke() {
        api::tests::smoke::<McsLock>();
    }
}
