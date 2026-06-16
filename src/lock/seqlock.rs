//! A sequence lock.
//! 顺序锁。

use core::mem;
use core::ops::Deref;
use core::sync::atomic::Ordering::*;
use core::sync::atomic::{fence, AtomicUsize};

use crossbeam_utils::Backoff;

/// A raw sequence lock.
/// 一个原始序列锁。
#[derive(Debug)]
pub struct RawSeqLock {
    /// - Even: unlocked or read-locked.
    /// - 即使：解锁或只读锁。
    /// - Odd: write-locked.
    /// - 奇数：写保护。
    /// - Is monotonically increasing. In particuler, a large part of the API is unsafe to enforce
    /// - 是单调递增的。特别是，该 API 的很大一部分在强制执行时是不安全的
    ///   this.
    /// 这个。
    seq: AtomicUsize,
}

impl Default for RawSeqLock {
    fn default() -> Self {
        Self::new()
    }
}

impl RawSeqLock {
    /// Creates a new raw sequence lock.
    /// 创建一个新的原始序列锁。
    pub const fn new() -> Self {
        Self {
            seq: AtomicUsize::new(0),
        }
    }

    /// Acquires a writer's lock.
    /// 获取写入者锁。
    pub fn write_lock(&self) -> usize {
        let backoff = Backoff::new();

        loop {
            let seq = self.seq.load(Relaxed);
            if seq & 1 == 0
                && self
                    .seq
                    .compare_exchange(seq, seq.wrapping_add(1), Acquire, Relaxed)
                    .is_ok()
            {
                fence(Release);
                return seq;
            }

            backoff.snooze();
        }
    }

    /// Releases a writer's lock.
    /// 释放写入者锁。
    ///
    /// # Safety
    /// # 安全
    ///
    /// - `self` must be a an acquired writer's lock.
    /// - `self` 必须是一个后天获得的作家锁。
    /// - `seq` must be from the most recent [`SeqLock::write_lock`] call on `self`.
    /// - `seq` 必须来自 `self` 上最近的 [`SeqLock::write_lock`] 调用。
    pub unsafe fn write_unlock(&self, seq: usize) {
        self.seq.store(seq.wrapping_add(2), Release);
    }

    /// Acquires a reader's lock.
    /// 获取读者锁。
    pub fn read_begin(&self) -> usize {
        let backoff = Backoff::new();

        loop {
            let seq = self.seq.load(Acquire);
            if seq & 1 == 0 {
                return seq;
            }

            backoff.snooze();
        }
    }

    /// Validates reads.
    /// 验证读取。
    ///
    /// If `self` is a read lock and `seq` is the corresponding sequence number,
    /// 如果 `self` 是一个读锁，而 `seq` 是相应的序列号，
    /// then if the return value is `true`, the reads are valid.
    /// 那么如果返回值是 `true`，这些读取是有效的。
    pub fn read_validate(&self, seq: usize) -> bool {
        fence(Acquire);

        seq == self.seq.load(Relaxed)
    }

    /// # Safety
    /// # 安全
    ///
    /// - `seq` must be even.
    /// - `seq` 必须是偶数。
    // No need to require `self` to be a read lock, as the sequence number is enough to validate.
    // 不需要要求 `self` 是读锁，因为序列号足以验证。
    pub unsafe fn upgrade(&self, seq: usize) -> bool {
        if self
            .seq
            .compare_exchange(seq, seq.wrapping_add(1), Acquire, Relaxed)
            .is_err()
        {
            return false;
        }

        fence(Release);
        true
    }
}

/// A sequence lock.
/// 顺序锁。
#[derive(Debug, Default)]
pub struct SeqLock<T> {
    inner: RawSeqLock,
    data: T,
}

/// A writer's lock guard.
/// 作家的锁护盖。
#[derive(Debug)]
pub struct WriteGuard<'s, T> {
    lock: &'s SeqLock<T>,
    seq: usize,
}

/// A reader's lock guard.
/// 读者的锁护罩。
#[derive(Debug)]
pub struct ReadGuard<'s, T> {
    lock: &'s SeqLock<T>,
    seq: usize,
}

// TODO: Think about the safety of these implementations.
// 待办：思考这些实现的安全性。
unsafe impl<T: Send> Send for SeqLock<T> {}
unsafe impl<T: Send + Sync> Sync for SeqLock<T> {}

unsafe impl<T> Send for WriteGuard<'_, T> {}
unsafe impl<T: Sync> Sync for WriteGuard<'_, T> {}

unsafe impl<T> Send for ReadGuard<'_, T> {}
unsafe impl<T: Sync> Sync for ReadGuard<'_, T> {}

impl<T> SeqLock<T> {
    /// Creates a new sequence lock.
    /// 创建一个新的序列锁。
    pub const fn new(data: T) -> Self {
        SeqLock {
            inner: RawSeqLock::new(),
            data,
        }
    }

    /// Consumes this seqlock, returning the underlying data.
    /// 消耗此序列锁，返回底层数据。
    pub fn into_inner(self) -> T {
        self.data
    }

    /// Dereferences the inner value.
    /// 解引用内部值。
    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }

    /// Acquires a writer's lock.
    /// 获取写入者锁。
    pub fn write_lock(&self) -> WriteGuard<T> {
        let seq = self.inner.write_lock();
        WriteGuard { lock: self, seq }
    }

    /// # Safety
    /// # 安全
    ///
    /// All reads from the underlying data should be atomic.
    /// 从底层数据的所有读取操作都应该是原子的。
    pub unsafe fn read_lock(&self) -> ReadGuard<T> {
        let seq = self.inner.read_begin();
        ReadGuard { lock: self, seq }
    }

    /// # Safety
    /// # 安全
    ///
    /// All reads from the underlying data should be atomic.
    /// 从底层数据的所有读取操作都应该是原子的。
    pub unsafe fn read<F, R>(&self, f: F) -> Option<R>
    where
        F: FnOnce(&T) -> R,
    {
        let guard = unsafe { self.read_lock() };
        let result = f(&guard);

        if guard.finish() {
            Some(result)
        } else {
            None
        }
    }
}

impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.lock.data
    }
}

impl<T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        // SAFETY:
        // 安全：
        //
        // - A `WriteGuard` implies `self.lock.inner` is an acquired write lock.
        // - A `WriteGuard` 意味着 `self.lock.inner` 是一个获取的写锁。
        // - `self.seq` is the proper sequence number of the write lock.
        // - `self.seq` 是写锁的正确序列号。
        unsafe { self.lock.inner.write_unlock(self.seq) };
    }
}

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.lock.data
    }
}

impl<T> Clone for ReadGuard<'_, T> {
    fn clone(&self) -> Self {
        Self {
            lock: self.lock,
            seq: self.seq,
        }
    }
}

impl<T> Drop for ReadGuard<'_, T> {
    fn drop(&mut self) {
        // HACK(@jeehoonkang): we really need linear type here:
        // HACK(@jeehoonkang)：我们这里真的需要线性类型：
        // https://github.com/rust-lang/rfcs/issues/814
        panic!("`seqlock::ReadGuard` should never drop. Use `ReadGuard::finish` instead.");
    }
}

impl<'s, T> ReadGuard<'s, T> {
    /// Validates reads.
    /// 验证读取。
    pub fn validate(&self) -> bool {
        self.lock.inner.read_validate(self.seq)
    }

    /// Restarts the read critical section.
    /// 重新启动读取关键部分。
    pub fn restart(&mut self) {
        self.seq = self.lock.inner.read_begin();
    }

    /// Releases the reader's lock.
    /// 释放阅读器的锁。
    pub fn finish(self) -> bool {
        let result = self.validate();
        mem::forget(self);
        result
    }

    /// Tries to upgrade to a writer's lock.
    /// 尝试升级到写入者锁。
    pub fn upgrade(self) -> Result<WriteGuard<'s, T>, ()> {
        // SAFETY:
        // 安全：
        //
        // - `self.seq` is the proper sequence number of the read lock, hence even.
        // - `self.seq` 是读锁的正确序列号，因此是偶数。
        let result = if unsafe { self.lock.inner.upgrade(self.seq) } {
            Ok(WriteGuard {
                lock: self.lock,
                seq: self.seq,
            })
        } else {
            Err(())
        };
        mem::forget(self);
        result
    }
}
