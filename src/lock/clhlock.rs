use core::sync::atomic::Ordering::*;
use core::sync::atomic::{AtomicBool, AtomicPtr};

use crossbeam_utils::{Backoff, CachePadded};

use crate::lock::*;

struct Node {
    locked: AtomicBool,
}

#[derive(Debug, Clone)]
pub struct Token(*const CachePadded<Node>);

// SAFETY: It doesn't matter if a thread used a token made by another thread.
// 安全性：线程使用另一个线程生成的令牌没有关系。
unsafe impl Send for Token {}

/// CLH lock.
/// CLH 锁
#[derive(Debug)]
pub struct ClhLock {
    tail: AtomicPtr<CachePadded<Node>>,
}

impl Node {
    fn new(locked: bool) -> *mut CachePadded<Self> {
        Box::into_raw(Box::new(CachePadded::new(Self {
            locked: AtomicBool::new(locked),
        })))
    }
}

impl Default for ClhLock {
    fn default() -> Self {
        let node = AtomicPtr::new(Node::new(false));

        Self { tail: node }
    }
}

unsafe impl RawLock for ClhLock {
    type Token = Token;

    fn lock(&self) -> Self::Token {
        let node = Node::new(true);
        let prev = self.tail.swap(node, AcqRel);
        let backoff = Backoff::new();

        // SAFETY: `prev` is valid, as `self.tail` was valid at initialization and any `swap()` to
        // 安全性：`prev` 是有效的，因为 `self.tail` 在初始化时有效，并且任何 `swap()`
        // it by other `lock()`s. Hence, it points to valid memory as the thread that made `prev`
        // 它被其他 `lock()` 使用。因此，它指向有效的内存，就像执行 `prev` 的线程一样
        // will not free it.
        // 不会释放它。
        while unsafe { (*prev).locked.load(Acquire) } {
            backoff.snooze();
        }

        // SAFETY: since `prev` was obtained from a swap on tail, only this thread other than its
        // 安全性：由于 `prev` 是从尾部交换获得的，除了它之外，只有这个线程
        // creator can access it. Since the creator will no longer access `prev` as its `locked` is
        // 创作者可以访问它。由于创作者将不再访问 `prev`，因为它的 `locked` 是
        // false, we have unique access to it.
        // 不，我们有独特的访问权限。
        drop(unsafe { Box::from_raw(prev) });
        Token(node)
    }

    unsafe fn unlock(&self, token: Self::Token) {
        unsafe { (*token.0).locked.store(false, Release) };
    }
}

impl Drop for ClhLock {
    fn drop(&mut self) {
        // Drop the node made by the last thread that `lock()`ed.
        // 删除最后一个由 `lock()`ed 的线程创建的节点。
        let node = *self.tail.get_mut();

        // SAFETY: Since this is the tail node, no other thread has access to it.
        // 安全：由于这是尾节点，没有其他线程可以访问它。
        drop(unsafe { Box::from_raw(node) });
    }
}

#[cfg(test)]
mod tests {
    use super::super::api;
    use super::ClhLock;

    #[test]
    fn smoke() {
        api::tests::smoke::<ClhLock>();
    }
}
