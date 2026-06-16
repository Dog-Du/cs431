use core::marker::PhantomData;
#[cfg(not(feature = "check-loom"))]
use core::sync::atomic::{Ordering, fence};

#[cfg(feature = "check-loom")]
use loom::sync::atomic::{Ordering, fence};

use super::{HAZARDS, HazardBag};

type Retired = (*mut (), unsafe fn(*mut ()));

/// Thread-local list of retired pointers.
/// 线程本地的已退役指针列表。
#[derive(Debug)]
pub struct RetiredSet<'s> {
    hazards: &'s HazardBag,
    /// The first element of the pair is the machine representation of the pointer and the second
    /// 该对的第一个元素是指针的机器表示，第二个元素是指针的第二个
    /// is the function pointer to `free::<T>` where `T` is the type of the object.
    /// 是指向 `free::<T>` 的函数指针，其中 `T` 是该对象的类型。
    inner: Vec<Retired>,
    _marker: PhantomData<*const ()>, // !Send + !Sync
    // !发送   !同步
}

impl<'s> RetiredSet<'s> {
    /// The max length of retired pointer list. `collect` is triggered when `THRESHOLD` pointers
    /// 已退休指针列表的最大长度。当有 `THRESHOLD` 个指针时触发 `collect`
    /// are retired.
    /// 已经退休。
    const THRESHOLD: usize = 64;

    /// Create a new retired pointer list protected by the given `HazardBag`.
    /// 创建一个由指定 `HazardBag` 保护的新已退役指针列表。
    pub fn new(hazards: &'s HazardBag) -> Self {
        Self {
            hazards,
            inner: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// Retires a pointer.
    /// 释放指针。
    ///
    /// # Safety
    /// # 安全
    ///
    /// * `pointer` must be removed from shared memory before calling this function, and must be
    /// * 在调用此函数之前，必须从共享内存中移除 `pointer`，并且必须
    ///   valid.
    /// 有效。
    /// * The same `pointer` should only be retired once.
    /// * 相同的 `pointer` 只应被退役一次。
    ///
    /// # Note
    /// # 注意
    ///
    /// `T: Send` is not required because the retired pointers are not sent to other threads.
    /// `T: Send` 不是必须的，因为被回收的指针不会发送到其他线程。
    pub unsafe fn retire<T>(&mut self, pointer: *mut T) {
        /// Frees a pointer. This function is defined here instead of `collect()` as we know about
        /// 释放一个指针。这个函数在这里定义，而不是在 `collect()` 中定义，因为我们对此有所了解
        /// the type of `pointer` only at the time of retiring it.
        /// 只有在退役它的时候，`pointer` 的类型。
        ///
        /// # Safety
        /// # 安全
        ///
        /// * Subsumes the safety requirements of [`Box::from_raw`]. In particular, one must have
        /// * 包含 [`Box::from_raw`] 的安全要求。特别是，必须具备
        ///   unique ownership to `data`.
        /// 将独特所有权转移到 `data`。
        ///
        /// [`Box::from_raw`]: https://doc.rust-lang.org/std/boxed/struct.Box.html#method.from_raw
        unsafe fn free<T>(data: *mut ()) {
            drop(unsafe { Box::from_raw(data.cast::<T>()) })
        }

        todo!()
    }

    /// Free the pointers that are `retire`d by the current thread and not `protect`ed by any other
    /// 释放当前线程 `retire`d 而未被其他任何线程 `protect`ed 的指针
    /// threads.
    /// 线程。
    pub fn collect(&mut self) {
        todo!()
    }
}

impl Default for RetiredSet<'static> {
    fn default() -> Self {
        Self::new(&HAZARDS)
    }
}

// this triggers loom internal bug
// 这会触发 Loom 的内部 bug
#[cfg(not(feature = "check-loom"))]
impl Drop for RetiredSet<'_> {
    fn drop(&mut self) {
        // In a production-quality implementation of hazard pointers, the remaining local retired
        // 在生产质量的 Hazard Pointer 实现中，其余的本地已退休
        // pointers will be moved to a global list of retired pointers, which are then reclaimed by
        // 指针将被移到已废弃指针的全局列表中，然后由...回收
        // the other threads. For pedagogical purposes, here we simply wait for all retired pointers
        // 其他线程。出于教学目的，这里我们只是等待所有已退役的指针
        // are no longer protected.
        // 不再受保护。
        while !self.inner.is_empty() {
            self.collect();
        }
    }
}

#[cfg(all(test, not(feature = "check-loom")))]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashSet;
    use std::rc::Rc;

    use super::{HazardBag, RetiredSet};

    // retire `THRESHOLD` pointers to trigger collection
    // 撤销 `THRESHOLD` 指针以触发收集
    #[test]
    fn retire_threshold_collect() {
        struct Tester(Rc<RefCell<HashSet<usize>>>, usize);
        impl Drop for Tester {
            fn drop(&mut self) {
                let _ = self.0.borrow_mut().insert(self.1);
            }
        }
        let hazards = HazardBag::new();
        let mut retires = RetiredSet::new(&hazards);
        let freed = Rc::new(RefCell::new(HashSet::new()));
        for i in 0..RetiredSet::THRESHOLD {
            unsafe { retires.retire(Box::leak(Box::new(Tester(freed.clone(), i)))) };
        }
        let freed = Rc::try_unwrap(freed).unwrap().into_inner();

        assert_eq!(freed, (0..RetiredSet::THRESHOLD).collect())
    }
}
