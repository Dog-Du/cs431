use core::ptr::{self, NonNull};
#[cfg(not(feature = "check-loom"))]
use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering, fence};
use std::collections::HashSet;
use std::fmt;

#[cfg(feature = "check-loom")]
use loom::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize, Ordering, fence};

use super::HAZARDS;

/// Represents the ownership of a hazard pointer slot.
/// 表示Hazard Pointer槽的所有权。
pub struct Shield {
    slot: NonNull<HazardSlot>,
}

impl Shield {
    /// Creates a new shield for hazard pointer.
    /// 为Hazard Pointer创建一个新的屏障。
    pub fn new(hazards: &HazardBag) -> Self {
        let slot = hazards.acquire_slot().into();
        Self { slot }
    }

    /// Store `pointer` to the hazard slot.
    /// 将 `pointer` 存入危险槽。
    pub fn set<T>(&self, pointer: *mut T) {
        todo!()
    }

    /// Clear the hazard slot.
    /// 清除危险槽。
    pub fn clear(&self) {
        self.set(ptr::null_mut::<()>())
    }

    /// Check if `src` still points to `pointer`. If not, returns the current value.
    /// 检查 `src` 是否仍然指向 `pointer`。如果没有，则返回当前值。
    ///
    /// For a pointer `p`, if "`src` still pointing to `pointer`" implies that `p` is not retired,
    /// 对于一个指针 `p`，如果“`src` 仍然指向 `pointer`”意味着 `p` 没有被废弃，
    /// then `Ok(())` means that shields set to `p` are validated.
    /// 那么 `Ok(())` 意味着设置为 `p` 的盾牌已被验证。
    pub fn validate<T>(pointer: *mut T, src: &AtomicPtr<T>) -> Result<(), *mut T> {
        todo!()
    }

    /// Try protecting `pointer` obtained from `src`. If not, returns the current value.
    /// 尝试保护从 `src` 获取的 `pointer`。如果不行，则返回当前值。
    ///
    /// If "`src` still pointing to `pointer`" implies that `pointer` is not retired, then `Ok(())`
    /// 如果“`src`仍然指向`pointer`”意味着`pointer`尚未退役，那么`Ok(())`
    /// means that this shield is validated.
    /// 意味着这个盾牌已被验证。
    pub fn try_protect<T>(&self, pointer: *mut T, src: &AtomicPtr<T>) -> Result<(), *mut T> {
        self.set(pointer);
        Self::validate(pointer, src).inspect_err(|_| self.clear())
    }

    /// Get a protected pointer from `src`.
    /// 从 `src` 获取一个受保护的指针。
    ///
    /// See `try_protect()`.
    /// 请参见 `try_protect()`。
    pub fn protect<T>(&self, src: &AtomicPtr<T>) -> *mut T {
        let mut pointer = src.load(Ordering::Relaxed);
        while let Err(new) = self.try_protect(pointer, src) {
            pointer = new;
            #[cfg(feature = "check-loom")]
            loom::sync::atomic::spin_loop_hint();
        }
        pointer
    }
}

impl Default for Shield {
    fn default() -> Self {
        Self::new(&HAZARDS)
    }
}

impl Drop for Shield {
    /// Clear and release the ownership of the hazard slot.
    /// 清除并释放危险槽的所有权。
    fn drop(&mut self) {
        todo!()
    }
}

impl fmt::Debug for Shield {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Shield")
            .field("slot address", &self.slot)
            .field("slot data", unsafe { self.slot.as_ref() })
            .finish()
    }
}

/// Global bag (multiset) of hazards pointers.
/// 全局Hazard Pointer包（多重集）。
/// `HazardBag.head` and `HazardSlot.next` form a grow-only list of all hazard slots. Slots are
/// `HazardBag.head` 和 `HazardSlot.next` 形成了一个只增不减的所有危险槽列表。槽位是
/// never removed from this list. Instead, it gets deactivated and recycled for other `Shield`s.
/// 从未从此列表中移除。相反，它会被停用并回收用于其他 `Shield`。
#[derive(Debug)]
pub struct HazardBag {
    head: AtomicPtr<HazardSlot>,
}

/// See `HazardBag`
/// 查看 `HazardBag`
#[derive(Debug)]
struct HazardSlot {
    // Whether this slot is occupied by a `Shield`.
    // 这个插槽是否被 `Shield` 占用。
    active: AtomicBool,
    // Machine representation of the hazard pointer.
    // Hazard Pointer的机器表示。
    hazard: AtomicPtr<()>,
    // Immutable pointer to the next slot in the bag.
    // 指向袋中下一个槽的不可变指针。
    next: *const HazardSlot,
}

impl HazardSlot {
    fn new() -> Self {
        todo!()
    }
}

impl HazardBag {
    #[cfg(not(feature = "check-loom"))]
    /// Creates a new global hazard set.
    /// 创建一个新的全局危险集合。
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    #[cfg(feature = "check-loom")]
    /// Creates a new global hazard set.
    /// 创建一个新的全局危险集合。
    pub fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Acquires a slot in the hazard set, either by recycling an inactive slot or allocating a new
    /// 在危险集合中获取一个槽位，可以通过回收一个不活跃的槽位或分配一个新槽位
    /// slot.
    /// 槽.
    fn acquire_slot(&self) -> &HazardSlot {
        todo!()
    }

    /// Find an inactive slot and activate it.
    /// 找到一个未使用的插槽并将其激活。
    fn try_acquire_inactive(&self) -> Option<&HazardSlot> {
        todo!()
    }

    /// Returns all the hazards in the set.
    /// 返回集合中的所有危险。
    pub fn all_hazards(&self) -> HashSet<*mut ()> {
        todo!()
    }
}

impl Default for HazardBag {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for HazardBag {
    /// Frees all slots.
    /// 释放所有槽位。
    fn drop(&mut self) {
        todo!()
    }
}

unsafe impl Send for HazardSlot {}
unsafe impl Sync for HazardSlot {}

#[cfg(all(test, not(feature = "check-loom")))]
mod tests {
    use std::collections::HashSet;
    use std::ops::Range;
    use std::sync::Arc;
    use std::sync::atomic::AtomicPtr;
    use std::{mem, thread};

    use super::{HazardBag, Shield};

    const THREADS: usize = 8;
    const VALUES: Range<usize> = 1..1024;

    // `all_hazards` should return hazards protected by shield(s).
    // `all_hazards` 应返回被护盾保护的危害。
    #[test]
    fn all_hazards_protected() {
        let hazard_bag = Arc::new(HazardBag::new());
        (0..THREADS)
            .map(|_| {
                let hazard_bag = hazard_bag.clone();
                thread::spawn(move || {
                    for data in VALUES {
                        let src = AtomicPtr::new(data as *mut ());
                        let shield = Shield::new(&hazard_bag);
                        let _ = shield.protect(&src);
                        // leak the shield so that it is not unprotected.
                        // 泄露护盾，以便它不会无保护。
                        mem::forget(shield);
                    }
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|th| th.join().unwrap());
        let all = hazard_bag.all_hazards();
        let values = VALUES.map(|data| data as *mut ()).collect();
        assert!(all.is_superset(&values))
    }

    // `all_hazards` should not return values that are no longer protected.
    // `all_hazards` 不应返回不再受保护的值。
    #[test]
    fn all_hazards_unprotected() {
        let hazard_bag = Arc::new(HazardBag::new());
        (0..THREADS)
            .map(|_| {
                let hazard_bag = hazard_bag.clone();
                thread::spawn(move || {
                    for data in VALUES {
                        let src = AtomicPtr::new(data as *mut ());
                        let shield = Shield::new(&hazard_bag);
                        let _ = shield.protect(&src);
                    }
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .for_each(|th| th.join().unwrap());
        let all = hazard_bag.all_hazards();
        let values = VALUES.map(|data| data as *mut ()).collect();
        let intersection: HashSet<_> = all.intersection(&values).collect();
        assert!(intersection.is_empty())
    }

    // `acquire_slot` should recycle existing slots.
    // `acquire_slot` 应该回收现有的时隙。
    #[test]
    fn recycle_slots() {
        let hazard_bag = HazardBag::new();
        // allocate slots
        // 分配时段
        let shields = (0..1024)
            .map(|_| Shield::new(&hazard_bag))
            .collect::<Vec<_>>();
        // slot addresses
        // 插槽地址
        let old_slots = shields
            .iter()
            .map(|s| s.slot.as_ptr() as usize)
            .collect::<HashSet<_>>();
        // release the slots
        // 释放插槽
        drop(shields);

        let shields = (0..128)
            .map(|_| Shield::new(&hazard_bag))
            .collect::<Vec<_>>();
        let new_slots = shields
            .iter()
            .map(|s| s.slot.as_ptr() as usize)
            .collect::<HashSet<_>>();

        // no new slots should've been created
        // 不应该创建新的插槽
        assert!(new_slots.is_subset(&old_slots));
    }
}
