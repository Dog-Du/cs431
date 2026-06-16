//! Hazard pointers.
//! Hazard Pointer
//!
//! # Example
//! # 示例
//!
//! ```
//! use std::ptr;
//! use std::sync::atomic::{AtomicPtr, Ordering};
//! use cs431_homework::hazard_pointer::{collect, retire, Shield};
//!
//! let shield = Shield::default();
//! let atomic = AtomicPtr::new(Box::leak(Box::new(1usize)));
//! let protected = shield.protect(&atomic);
//! assert_eq!(unsafe { *protected }, 1);
//!
//! // unlink the block and retire
//! atomic.store(ptr::null_mut(), Ordering::Relaxed);
//! unsafe { retire(protected); }
//!
//! // manually trigger reclamation (not necessary)
//! collect();
//! ```

use core::cell::RefCell;
#[cfg(not(feature = "check-loom"))]
use std::thread_local;

#[cfg(feature = "check-loom")]
use loom::thread_local;

mod hazard;
mod retire;

pub use hazard::{HazardBag, Shield};
pub use retire::RetiredSet;

#[cfg(not(feature = "check-loom"))]
/// Default global bag of all hazard pointers.
/// 所有Hazard Pointer的默认全局包。
pub static HAZARDS: HazardBag = HazardBag::new();

#[cfg(feature = "check-loom")]
// FIXME: loom does not currently provide the equivalent of Lazy:
// FIXME：loom 目前不提供 Lazy 的等效功能：
// https://github.com/tokio-rs/loom/issues/263
loom::lazy_static! {
    /// Default global bag of all hazard pointers.
    /// 所有Hazard Pointer的默认全局包。
    pub static ref HAZARDS: HazardBag = HazardBag::new();
}

thread_local! {
    /// Default thread-local retired pointer list.
    /// 默认线程本地已退役指针列表。
    static RETIRED: RefCell<RetiredSet<'static>> = RefCell::new(RetiredSet::default());
}

/// Retires a pointer.
/// 释放指针。
///
/// # Safety
/// # 安全
///
/// * `pointer` must be removed from shared memory before calling this function, and must be valid.
/// * 在调用此函数之前，必须从共享内存中删除 `pointer`，并且必须是有效的。
/// * The same `pointer` should only be retired once.
/// * 相同的 `pointer` 只应被退役一次。
pub unsafe fn retire<T>(pointer: *mut T) {
    RETIRED.with(|r| unsafe { r.borrow_mut().retire(pointer) });
}

/// Frees the pointers that are `retire`d by the current thread and not `protect`ed by any other
/// 释放当前线程所 `retire`d 的指针，而未被任何其他线程 `protect`ed
/// threads.
/// 线程。
pub fn collect() {
    RETIRED.with(|r| r.borrow_mut().collect());
}
