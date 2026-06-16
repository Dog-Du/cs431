use core::marker::PhantomData;
use core::mem::ManuallyDrop;
use core::ops::Deref;
use std::time;

use crossbeam_epoch::{Atomic, Guard, Owned, pin};
use rand::{self, Rng};

pub(crate) const ELIM_SIZE: usize = 16;
pub(crate) const ELIM_DELAY: time::Duration = time::Duration::from_millis(10);

#[inline]
pub(crate) fn get_random_elim_index() -> usize {
    (rand::rng().random::<u64>() as usize) % ELIM_SIZE
}

/// Concurrent stack types.
/// 并发栈类型。
pub trait Stack<T>: Default {
    /// Push request type.
    /// 推送请求类型。
    type PushReq: From<T> + Deref<Target = ManuallyDrop<T>>;

    /// Tries to push a value to the stack.
    /// 尝试将一个值压入栈中。
    ///
    /// Returns `Ok(())` if the push request is served; `Err(req)` is CAS failed.
    /// 如果推送请求已处理，则返回 `Ok(())`；CAS 失败则返回 `Err(req)`。
    fn try_push(
        &self,
        req: Owned<Self::PushReq>,
        guard: &Guard,
    ) -> Result<(), Owned<Self::PushReq>>;

    /// Tries to pop a value from the stack.
    /// 尝试从栈中弹出一个值。
    ///
    /// Returns `Ok(Some(v))` if `v` is popped; `Ok(None)` if the stack is empty; and `Err(())` if
    /// 如果弹出 `v`，则返回 `Ok(Some(v))`；如果栈为空，则返回 `Ok(None)`；如果
    /// CAS failed.
    /// CAS 失败。
    fn try_pop(&self, guard: &Guard) -> Result<Option<T>, ()>;

    /// Returns `true` if the stack is empty.
    /// 如果堆栈为空，则返回 `true`。
    fn is_empty(&self, guard: &Guard) -> bool;

    /// Pushes a value to the stack.
    /// 将一个值压入栈中。
    fn push(&self, t: T) {
        let mut req = Owned::new(t.into());
        let guard = pin();
        while let Err(r) = self.try_push(req, &guard) {
            req = r;
        }
    }

    /// Pops a value from the stack.
    /// 从栈中弹出一个值。
    ///
    /// Returns `Some(v)` if `v` is popped; `None` if the stack is empty.
    /// 如果弹出 `v`，则返回 `Some(v)`；如果栈为空，则返回 `None`。
    fn pop(&self) -> Option<T> {
        let guard = pin();
        loop {
            if let Ok(result) = self.try_pop(&guard) {
                return result;
            }
        }
    }
}

#[derive(Debug)]
pub struct ElimStack<T, S: Stack<T>> {
    pub(crate) inner: S,
    // slot tags:
    // 插槽标签：
    // - 0: no request
    // - 0：无请求
    // - 1: push request
    // - 1：推送请求
    // - 2: pop request
    // - 2：弹出请求
    // - 3: request acknowledged
    // - 3：请求已确认
    pub(crate) slots: [Atomic<S::PushReq>; ELIM_SIZE],
}

impl<T, S: Stack<T>> Default for ElimStack<T, S> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            slots: Default::default(),
        }
    }
}
