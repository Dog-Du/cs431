use core::cell::UnsafeCell;
use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};

/// Raw lock interface.
/// 原始锁接口。
///
/// # Safety
/// # 安全
///
/// Implementations of this trait must ensure that the lock is actually exclusive: a lock can't be
/// 此特性的实现必须确保锁实际上是独占的：锁不能被
/// acquired while the lock is already locked.
/// 在锁已经锁住的情况下获取的。
// TODO: For weak memory, there needs to be a bit more stricter condition. unlock -hb→ lock.
// 待办事项：对于弱内存，需要一个更严格的条件。unlock -hb→ lock。
pub unsafe trait RawLock: Default + Send + Sync {
    /// Raw lock's token type.
    /// 原始锁的令牌类型。
    ///
    /// We don't enforce `Send`/`Sync`, as some locks may not satisfy it. We restrict them at
    /// 我们不强制执行 `Send`/`Sync`，因为某些锁可能不符合要求。我们在...处限制它们
    /// `Send`/`Sync` impl for [`LockGuard`].
    /// `Send`/`Sync` 为 [`LockGuard`] 实现。
    type Token;

    /// Acquires the raw lock.
    /// 获取原始锁。
    fn lock(&self) -> Self::Token;

    /// Releases the raw lock.
    /// 释放原始锁。
    ///
    /// # Safety
    /// # 安全
    ///
    /// - `self` must be an acquired lock.
    /// - `self` 必须是一个已获取的锁。
    /// - `token` must be from a [`RawLock::lock`] or [`RawTryLock::try_lock`] call to `self`.
    /// - `token` 必须来自对 `self` 的 [`RawLock::lock`] 或 [`RawTryLock::try_lock`] 调用。
    unsafe fn unlock(&self, token: Self::Token);
}

/// Raw lock interface for the try_lock API.
/// 用于 try_lock API 的原始锁接口。
///
/// # Safety
/// # 安全
///
/// See [`RawLock`] for safety requirements.
/// 有关安全要求，请参见 [`RawLock`]。
///
/// Also, [`RawTryLock::try_lock`] should return a token that can be used for [`RawLock::unlock`].
/// 此外，[`RawTryLock::try_lock`] 应该返回一个可以用于 [`RawLock::unlock`] 的令牌。
pub unsafe trait RawTryLock: RawLock {
    /// Tries to acquire the raw lock.
    /// 尝试获取原始锁。
    fn try_lock(&self) -> Result<Self::Token, ()>;
}

/// A type-safe lock.
/// 类型安全锁。
#[derive(Debug, Default)]
pub struct Lock<L: RawLock, T> {
    inner: L,
    data: UnsafeCell<T>,
}

// Send is automatically implemented for Lock.
// Send 已为 Lock 自动实现。

// SATEFY: threads can only access `&mut T` via the lock, and `L` is `Sync`.
// 安全性：线程只能通过锁访问 `&mut T`，并且 `L` 是 `Sync`。
unsafe impl<L: RawLock, T: Send> Sync for Lock<L, T> {}

impl<L: RawLock, T> Lock<L, T> {
    /// Creates a new lock.
    /// 创建一个新的锁。
    pub fn new(data: T) -> Self {
        Self {
            inner: L::default(),
            data: UnsafeCell::new(data),
        }
    }

    /// Destroys the lock and retrieves the lock-protected value.
    /// 销毁锁并取回受锁保护的值。
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    /// Acquires the lock and dereferences the inner value.
    /// 获取锁并解引用内部值。
    pub fn lock(&self) -> LockGuard<L, T> {
        let token = self.inner.lock();
        LockGuard {
            lock: self,
            token: ManuallyDrop::new(token),
        }
    }
}

impl<L: RawTryLock, T> Lock<L, T> {
    /// Tries to acquire the lock and dereferences the inner value.
    /// 尝试获取锁并解引用内部值。
    pub fn try_lock(&self) -> Result<LockGuard<L, T>, ()> {
        self.inner.try_lock().map(|token| LockGuard {
            lock: self,
            token: ManuallyDrop::new(token),
        })
    }
}

/// A guard that holds the lock and dereferences the inner value.
/// 一个持有锁并解引用内部值的守卫。
#[derive(Debug)]
pub struct LockGuard<'s, L: RawLock, T> {
    lock: &'s Lock<L, T>,
    token: ManuallyDrop<L::Token>,
}

// Not auto derived as the auto-derived impls are incorrect. Remember, auto-derived impls are only
// 不是自动派生的，因为自动派生的实现是不正确的。请记住，自动派生的实现只是
// correct if there are no unsafe code used in the struct's methods.
// 如果结构体的方法中没有使用不安全代码，请更正。

// SAFETY: Ownership of `LockGuard` implies ownership of `L::Token` and `T`. Thus, they must both be
// 安全：拥有 `LockGuard` 意味着拥有 `L::Token` 和 `T`。因此，它们两者都必须
// `Send`.
// `Send`。
unsafe impl<L: RawLock, T: Send> Send for LockGuard<'_, L, T> where L::Token: Send {}

// SAFETY: Reference to `LockGuard` implies reference to `T`. Thus, `T` must be `Sync`.
// 安全：对 `LockGuard` 的引用意味着对 `T` 的引用。因此，`T` 必须是 `Sync`。
unsafe impl<L: RawLock, T: Sync> Sync for LockGuard<'_, L, T> {}

impl<L: RawLock, T> Drop for LockGuard<'_, L, T> {
    fn drop(&mut self) {
        // SAFETY: `self.token` is not used anymore in this function, and as we are `drop`ing
        // 安全：`self.token` 在此功能中不再使用，并且由于我们正在 `drop`
        // `self`, it is not used anymore.
        // `self`，它不再被使用。
        let token = unsafe { ManuallyDrop::take(&mut self.token) };

        // SAFETY: since `self` was created with `lock` and it's `token`, the `token` given to
        // 安全：由于 `self` 是用 `lock` 创建的，并且它是 `token`，提供给 `token` 的
        // `unlock()` is correct.
        // `unlock()` 是正确的。
        unsafe { self.lock.inner.unlock(token) };

        // Note: Important that nothing is done to `data` after `unlock()`.
        // 注意：重要的是在 `unlock()` 之后不要对 `data` 做任何事情。
    }
}

impl<L: RawLock, T> Deref for LockGuard<'_, L, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // 安全：
        // - Existance of a `LockGuard` means the lock is acquired, so the data is valid.
        // - `LockGuard` 的存在意味着锁已被获取，因此数据是有效的。
        // - Having a shared reference to the `LockGuard` implies there is no accessor making a
        // - 对 `LockGuard` 拥有共享引用意味着没有访问器创建一个
        //   mutable reference to the data.
        // 对数据的可变引用。
        unsafe { &*self.lock.data.get() }
    }
}

impl<L: RawLock, T> DerefMut for LockGuard<'_, L, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY:
        // 安全：
        // - Existance of a `LockGuard` means the lock is acquired, so the data is valid.
        // - `LockGuard` 的存在意味着锁已被获取，因此数据是有效的。
        // - Having a mutable reference to the `LockGuard` implies there is no accessor to data.
        // - 拥有对 `LockGuard` 的可变引用意味着没有访问数据的访问器。
        unsafe { &mut *self.lock.data.get() }
    }
}

#[cfg(test)]
pub mod tests {
    use std::thread::scope;

    use super::{Lock, RawLock};

    pub fn smoke<L: RawLock>() {
        const LENGTH: usize = 1024;
        let d = Lock::<L, Vec<usize>>::default();

        scope(|s| {
            let d = &d;
            for i in 1..LENGTH {
                s.spawn(move || {
                    let mut d = d.lock();
                    d.push(i);
                });
            }
        });

        let mut d = d.into_inner();
        d.sort_unstable();
        assert_eq!(d, (1..LENGTH).collect::<Vec<usize>>());
    }
}
