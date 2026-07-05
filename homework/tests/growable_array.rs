#![feature(cfg_sanitize)]

use core::ops::Deref;
use core::sync::atomic::Ordering::*;

use crossbeam_epoch::{Guard, Owned, Shared, pin};
use cs431_homework::test::adt::map;
use cs431_homework::{ConcurrentMap, GrowableArray};
use stack::{Node, Stack};

#[derive(Debug)]
struct ArrayMap<V> {
    array: GrowableArray<Node<V>>,
    /// dump everything into a stack and drop them later
    /// 把所有东西都堆到一个堆栈里，然后再丢掉它们
    storage: Stack<V>,
}

impl<V> Default for ArrayMap<V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<V> ArrayMap<V> {
    fn new() -> Self {
        Self {
            array: GrowableArray::new(),
            storage: Stack::new(),
        }
    }
}

/// Simple map implementation using the array index as the key.
/// 使用数组索引作为键的简单映射实现。
/// Uses u32 key instead of u64 to limit memory usage and runtime
/// 使用 u32 密钥而不是 u64 以限制内存使用和运行时间
impl<V> ConcurrentMap<u32, V> for ArrayMap<V> {
    fn lookup<'g>(&self, key: &u32, guard: &'g Guard) -> Option<&'g V> {
        let slot = self.array.get(*key as usize, guard);
        let ptr = slot.load(Acquire, guard);
        unsafe { ptr.as_ref() }.map(Deref::deref)
    }

    fn insert(&self, key: u32, value: V, guard: &Guard) -> Result<(), V> {
        let slot = self.array.get(key as usize, guard);
        let node = Owned::new(Node::new(value));
        match slot.compare_exchange(Shared::null(), node, AcqRel, Acquire, guard) {
            Ok(n) => {
                // Can't change `n` to `Owned` as it is in shared memory.
                // 无法将 `n` 更改为 `Owned`，因为它位于共享内存中。
                //
                // SAFETY: `n` is created in this function, hence this is the unique push of `n`.
                // 安全性：`n` 在此函数中被创建，因此这是 `n` 的唯一推送。
                // Also, `n` is not used again.
                // 此外，`n` 不会再次使用。
                unsafe { self.storage.push_node(n, guard) };
                Ok(())
            }
            Err(e) => Err(e.new.into_box().into_inner()),
        }
    }

    fn delete<'g>(&self, key: &u32, guard: &'g Guard) -> Result<&'g V, ()> {
        let slot = self.array.get(*key as usize, guard);
        let curr = slot.load(Relaxed, guard);
        // no entry
        // 禁止进入
        if curr.is_null() {
            return Err(());
        }
        match slot.compare_exchange(curr, Shared::null(), AcqRel, Acquire, guard) {
            Ok(_) => Ok(unsafe { curr.deref() }),
            Err(_) => Err(()), // already removed
                               // 已移除
        }
    }
}

mod stack {
    use core::cell::UnsafeCell;
    use core::ops::Deref;
    use core::sync::atomic::Ordering::*;
    use core::{mem, ptr};

    use crossbeam_epoch::{Atomic, Guard, Owned, Shared};

    #[derive(Debug)]
    pub(super) struct Stack<T> {
        head: Atomic<Node<T>>,
    }

    impl<T> Stack<T> {
        pub(super) fn new() -> Self {
            Self {
                head: Atomic::null(),
            }
        }
    }

    #[derive(Debug)]
    pub(super) struct Node<T> {
        data: T,
        next: UnsafeCell<*const Node<T>>,
    }

    impl<T> Node<T> {
        pub(super) fn new(data: T) -> Self {
            Self {
                data,
                next: UnsafeCell::new(ptr::null()),
            }
        }

        pub(super) fn into_inner(self) -> T {
            self.data
        }
    }

    impl<T> Deref for Node<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.data
        }
    }

    unsafe impl<T: Send> Send for Node<T> {}
    unsafe impl<T: Sync> Sync for Node<T> {}

    impl<T> Stack<T> {
        /// This stack is used as a temporary trash can for nodes. As such, unlike the Trieber's
        /// 这个栈被用作节点的临时垃圾桶。因此，与Trieber的不同
        /// stack in the lecture, we cannot require the full ownership of pushed nodes. So we mark
        /// 在讲座中的堆栈，我们不能要求被推入节点的完整所有权。所以我们标记
        /// it as `unsafe` to prevent the same node being pushed multiple times.
        /// 将其设置为 `unsafe`，以防止同一个节点被多次推送。
        ///
        /// # Safety
        /// # 安全
        ///
        /// - A single `n` should only be pushed into the stack once.
        /// - 一个 `n` 应该只被压入堆栈一次。
        /// - After the push, `n` should not be used again.
        /// - 推送后，不应再次使用 `n`。
        pub(super) unsafe fn push_node<'g>(&self, n: Shared<'g, Node<T>>, guard: &'g Guard) {
            let mut head = self.head.load(Relaxed, guard);
            loop {
                // SAEFTY: as n is pused only once, and after the push, n is not used again, we are
                // 安全性：由于 n 只被推送一次，并且在推送之后，n 不再被使用，我们是
                // the unique accessor of `n.next`. Hence non-atomic write is safe.
                // `n.next` 的唯一访问器。因此，非原子写是安全的。
                unsafe { *n.deref().next.get() = head.as_raw() };

                // TODO: Relaxed fine here? Might need release so that it syncs with `drop`?
                // 待办：这里可以放宽吗？可能需要发布，以便与 `drop` 同步？
                match self.head.compare_exchange(head, n, Relaxed, Relaxed, guard) {
                    Ok(_) => break,
                    Err(e) => head = e.current,
                }
            }
        }
    }

    impl<T> Drop for Stack<T> {
        fn drop(&mut self) {
            let mut o_curr = mem::take(&mut self.head);

            while let Some(curr) = unsafe { o_curr.try_into_owned() }.map(Owned::into_box) {
                o_curr = curr.next.into_inner().into();
            }
        }
    }
}

#[test]
fn smoke() {
    let list = ArrayMap::default();

    let guard = pin();

    assert_eq!(list.insert(37, 37, &guard), Ok(()));
    assert_eq!(list.lookup(&42, &guard), None);
    assert_eq!(list.lookup(&37, &guard), Some(&37));

    assert_eq!(list.insert(42, 42, &guard), Ok(()));
    assert_eq!(list.lookup(&42, &guard), Some(&42));
    assert_eq!(list.lookup(&37, &guard), Some(&37));

    assert_eq!(list.delete(&37, &guard), Ok(&37));
    assert_eq!(list.lookup(&42, &guard), Some(&42));
    assert_eq!(list.lookup(&37, &guard), None);

    assert_eq!(list.delete(&37, &guard), Err(()));
    assert_eq!(list.lookup(&42, &guard), Some(&42));
    assert_eq!(list.lookup(&37, &guard), None);
}

#[test]
fn stress_sequential() {
    const STEPS: usize = 4096;
    map::stress_sequential::<_, _, ArrayMap<usize>>(STEPS);
}

#[test]
fn lookup_concurrent() {
    const THREADS: usize = 4;
    const STEPS: usize = 4096;
    map::lookup_concurrent::<_, _, ArrayMap<usize>>(THREADS, STEPS);
}

#[test]
fn insert_concurrent() {
    const THREADS: usize = 8;
    const STEPS: usize = 4096 * 4;
    map::insert_concurrent::<_, _, ArrayMap<usize>>(THREADS, STEPS);
}

#[test]
fn stress_concurrent() {
    const THREADS: usize = if cfg!(sanitize = "thread") { 4 } else { 16 };
    const STEPS: usize = 4096 * if cfg!(sanitize = "thread") { 128 } else { 512 };
    map::stress_concurrent::<_, _, ArrayMap<usize>>(THREADS, STEPS);
}

#[test]
fn log_concurrent() {
    const THREADS: usize = if cfg!(sanitize = "thread") { 4 } else { 16 };
    const STEPS: usize = 4096 * if cfg!(sanitize = "thread") { 16 } else { 64 };
    map::log_concurrent::<_, _, ArrayMap<usize>>(THREADS, STEPS);
}
