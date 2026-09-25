//! Growable array.
//! 可增长数组。

use core::fmt::Debug;
use core::mem::{self, ManuallyDrop};
use core::sync::atomic::Ordering::*;

use crossbeam_epoch::{Atomic, Guard, Owned, Shared};

/// Growable array of `Atomic<T>`.
/// `Atomic<T>` 的可增长数组。
///
/// This is more complete version of the dynamic sized array from the paper. In the paper, the
/// 这是论文中动态大小数组的更完整版本。在论文中，
/// segment table is an array of arrays (segments) of pointers to the elements. In this
/// 段表是一个由数组（段）组成的数组，这些数组指向元素的指针。在这里
/// implementation, a segment contains the pointers to the elements **or other child segments**. In
/// 在实现中，一个段包含指向元素 **或其他子段** 的指针。
/// other words, it is a tree that has segments as internal nodes.
/// 换句话说，它是一棵以段作为内部节点的树。
///
/// # Example run
/// # 示例运行
///
/// Suppose `SEGMENT_LOGSIZE = 3` (segment size 8).
/// 假设 `SEGMENT_LOGSIZE = 3`（分段大小 8）。
///
/// When a new `GrowableArray` is created, `root` is initialized with `Atomic::null()`.
/// 当创建一个新的 `GrowableArray` 时，`root` 会用 `Atomic::null()` 初始化。
///
/// ```text
///                          +----+
///                          |root|
///                          +----+
/// ```
///
/// When you store element `cat` at the index `0b001`, it first initializes a segment.
/// 当你在索引 `0b001` 存储元素 `cat` 时，它首先会初始化一个段。
///
/// ```text
///                          +----+
///                          |root|
///                          +----+
///                            | height: 1
///                            v
///                 +---+---+---+---+---+---+---+---+
///                 |111|110|101|100|011|010|001|000|
///                 +---+---+---+---+---+---+---+---+
///                                           |
///                                           v
///                                         +---+
///                                         |cat|
///                                         +---+
/// ```
///
/// When you store `fox` at `0b111011`, it is clear that there is no room for indices larger than
/// 当你将 `fox` 存储在 `0b111011` 时，很明显没有空间存放大于的索引
/// `0b111`. So it first allocates another segment for upper 3 bits and moves the previous root
/// `0b111`。因此，它首先为高3位分配另一个段，并移动之前的根
/// segment (`0b000XXX` segment) under the `0b000XXX` branch of the the newly allocated segment.
/// 新分配段的 `0b000XXX` 分支下的段（`0b000XXX` 段）。
///
/// ```text
///                          +----+
///                          |root|
///                          +----+
///                            | height: 2
///                            v
///                 +---+---+---+---+---+---+---+---+
///                 |111|110|101|100|011|010|001|000|
///                 +---+---+---+---+---+---+---+---+
///                                               |
///                                               v
///                                      +---+---+---+---+---+---+---+---+
///                                      |111|110|101|100|011|010|001|000|
///                                      +---+---+---+---+---+---+---+---+
///                                                                |
///                                                                v
///                                                              +---+
///                                                              |cat|
///                                                              +---+
/// ```
///
/// And then, it allocates another segment for `0b111XXX` indices.
/// 然后，它为 `0b111XXX` 索引分配另一个段。
///
/// ```text
///                          +----+
///                          |root|
///                          +----+
///                            | height: 2
///                            v
///                 +---+---+---+---+---+---+---+---+
///                 |111|110|101|100|011|010|001|000|
///                 +---+---+---+---+---+---+---+---+
///                   |                           |
///                   v                           v
/// +---+---+---+---+---+---+---+---+    +---+---+---+---+---+---+---+---+
/// |111|110|101|100|011|010|001|000|    |111|110|101|100|011|010|001|000|
/// +---+---+---+---+---+---+---+---+    +---+---+---+---+---+---+---+---+
///                   |                                            |
///                   v                                            v
///                 +---+                                        +---+
///                 |fox|                                        |cat|
///                 +---+                                        +---+
/// ```
///
/// Finally, when you store `owl` at `0b000110`, it traverses through the `0b000XXX` branch of the
/// 最后，当你将 `owl` 存储在 `0b000110` 时，它会通过 `0b000XXX` 分支
/// height 2 segment and arrives at its `0b110` leaf.
/// 高度2段并到达其`0b110`叶子。
///
/// ```text
///                          +----+
///                          |root|
///                          +----+
///                            | height: 2
///                            v
///                 +---+---+---+---+---+---+---+---+
///                 |111|110|101|100|011|010|001|000|
///                 +---+---+---+---+---+---+---+---+
///                   |                           |
///                   v                           v
/// +---+---+---+---+---+---+---+---+    +---+---+---+---+---+---+---+---+
/// |111|110|101|100|011|010|001|000|    |111|110|101|100|011|010|001|000|
/// +---+---+---+---+---+---+---+---+    +---+---+---+---+---+---+---+---+
///                   |                        |                   |
///                   v                        v                   v
///                 +---+                    +---+               +---+
///                 |fox|                    |owl|               |cat|
///                 +---+                    +---+               +---+
/// ```
///
/// When the array is dropped, only the segments are dropped and the **elements must not be
/// 当数组被丢弃时，只有片段会被丢弃，**元素不能被
/// dropped/deallocated**.
/// 已释放/已取消分配**。
///
/// ```text
///                 +---+                    +---+               +---+
///                 |fox|                    |owl|               |cat|
///                 +---+                    +---+               +---+
/// ```
///
/// Instead, it should be handled by the container that the elements actually belong to. For
/// 相反，它应该由这些元素实际所属的容器来处理。对于
/// example, in `SplitOrderedList` the destruction of elements are handled by the inner `List`.
/// 例如，在 `SplitOrderedList` 中，元素的销毁由内部的 `List` 处理。
#[derive(Debug)]
pub struct GrowableArray<T> {
    root: Atomic<Segment<T>>,
}

const SEGMENT_LOGSIZE: usize = 10;

/// A fixed size array of atomic pointers to other `Segment<T>` or `T`.
/// 一个固定大小的数组，包含指向其他 `Segment<T>` 或 `T` 的原子指针。
///
/// Each segment is either an inner segment with pointers to other, children `Segment<T>` or an
/// 每个段都是一个内部段，带有指向其他子 `Segment<T>` 的指针，或者是一个
/// element segment with pointers to `T`. This is determined by the height of this segment in the
/// 元素段，包含指向 `T` 的指针。这由该段的高度决定
/// main array, which one needs to track separately. For example, use the main array root's tag.
/// 主数组，需要单独跟踪的那个。例如，使用主数组根的标签。
///
/// Since destructing segments requires its height information, it is not recommended to implement
/// 由于销毁段需要其高度信息，因此不建议实现
/// [`Drop`]. Rather, implement and use the custom [`Segment::deallocate`] method that accounts for
/// [`Drop`]。相反，应实现并使用考虑到的自定义 [`Segment::deallocate`] 方法
/// the height of the segment.
/// 该线段的高度。
union Segment<T> {
    children: ManuallyDrop<[Atomic<Segment<T>>; 1 << SEGMENT_LOGSIZE]>,
    elements: ManuallyDrop<[Atomic<T>; 1 << SEGMENT_LOGSIZE]>,
}

impl<T> Segment<T> {
    /// Create a new segment filled with null pointers. It is up to the callee to whether to use
    /// 创建一个填充有空指针的新段。是否使用由被调用者决定
    /// this as an intermediate or an element segment.
    /// 这作为一个中间段或一个元素段。
    fn new() -> Owned<Self> {
        Owned::new(
            // SAFETY: An array of null pointers can be interperted as either an intermediate
            // 安全性：一组空指针可以被解释为中间值之一
            // segment or an element segment.
            // 段或元素段。
            unsafe { mem::zeroed() },
        )
    }

    /// Deallocates a segment of `height`.
    /// 释放 `height` 的一个段。
    ///
    /// # Safety
    /// # 安全
    ///
    /// - `self` must actually have height `height`.
    /// - `self` 实际上必须有高度 `height`。
    /// - There should be no other references to possible children segments.
    /// - 不应有其他对可能的子段的引用。
    unsafe fn deallocate(self, height: usize) {
        debug_assert!(height > 0);

        if height > 1 {
            // 只有内部段的槽位才指向子段。叶子段的槽位指向 T，而 T 的所有权属于
            // 使用 GrowableArray 的外层容器，因此这里绝不能释放那些元素。
            let children = unsafe { ManuallyDrop::into_inner(self.children) };

            for child in children {
                // 此时 GrowableArray 已经被独占地析构，不会再有线程访问这些段，
                // 所以可以把 Atomic 中的指针重新取回为 Owned。
                if let Some(child) = unsafe { child.try_into_owned() } {
                    // 子段比当前段低一层；递归时必须携带这个高度，才能正确解释 union。
                    unsafe { child.into_box().deallocate(height - 1) };
                }
            }
        }
    }
}

impl<T> Debug for Segment<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Segment")
    }
}

impl<T> Drop for GrowableArray<T> {
    /// Deallocate segments, but not the individual elements.
    /// 释放段，但不要释放单个元素。
    fn drop(&mut self) {
        // 用空指针替换根，取得整棵段树的唯一所有权。
        let root = mem::take(&mut self.root);

        // SAFETY: drop 拿到了 &mut self，此时外部不能再通过数组取得新的段引用。
        // GrowableArray 的使用约定也要求销毁前已经结束所有并发访问。
        if let Some(root) = unsafe { root.try_into_owned() } {
            let height = root.tag();
            debug_assert!(height > 0);

            // 根指针的 tag 记录树高；实际的递归释放由 Segment::deallocate 完成。
            unsafe { root.into_box().deallocate(height) };
        }
    }
}

impl<T> Default for GrowableArray<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> GrowableArray<T> {
    /// Create a new growable array.
    /// 创建一个新的可增长数组。
    pub fn new() -> Self {
        Self {
            root: Atomic::null(),
        }
    }

    /// Returns the reference to the `Atomic` pointer at `index`. Allocates new segments if
    /// 返回位于 `index` 的 `Atomic` 指针的引用。如果需要则分配新的段
    /// necessary.
    /// 必要的。
    pub fn get<'g>(&self, index: usize, guard: &'g Guard) -> &'g Atomic<T> {
        const SEGMENT_SIZE: usize = 1 << SEGMENT_LOGSIZE;
        const SEGMENT_MASK: usize = SEGMENT_SIZE - 1;

        // 每层负责索引中的 SEGMENT_LOGSIZE 位。索引 0 也至少需要一个叶子段。
        let significant_bits = usize::BITS as usize - index.leading_zeros() as usize;
        let required_height = significant_bits.max(1).div_ceil(SEGMENT_LOGSIZE);

        let mut root = self.root.load(SeqCst, guard);

        // 根的 tag 保存当前树高。树不够高时，每次在旧根上方增加一层；旧根覆盖的
        // 都是低位索引，所以它应当成为新根的第 0 个子段。
        while root.tag() < required_height {
            let old_height = root.tag();
            let new_root = Segment::new();

            if !root.is_null() {
                // SAFETY: new_root 尚未发布，当前线程独占它，可以直接初始化 union 的
                // children 视图。子指针不能携带根的高度 tag。
                unsafe {
                    new_root.children[0].store(root.with_tag(0), SeqCst);
                }
            }

            match self.root.compare_exchange(
                root,
                new_root.with_tag(old_height + 1),
                SeqCst,
                SeqCst,
                guard,
            ) {
                // compare_exchange 成功时返回刚发布的新根。
                Ok(installed) => root = installed,
                // 失败说明别的线程先扩展了根；e.new 会在离开此分支时自动释放。
                Err(e) => root = e.current,
            }
        }

        debug_assert!(!root.is_null());

        let mut height = root.tag();
        // 高度只属于根指针；进入树后，所有子指针都使用 tag 0。
        let mut segment = root.with_tag(0);

        while height > 1 {
            // 从高到低，每层取索引中的一组 SEGMENT_LOGSIZE 位作为子段下标。
            let shift = (height - 1) * SEGMENT_LOGSIZE;
            let child_index = (index >> shift) & SEGMENT_MASK;

            // SAFETY: height > 1 说明当前段是内部段，因此应按 children 解释 union。
            let child_slot = unsafe { &segment.deref().children[child_index] };
            let mut child = child_slot.load(SeqCst, guard);

            if child.is_null() {
                match child_slot.compare_exchange(
                    Shared::null(),
                    Segment::new(),
                    SeqCst,
                    SeqCst,
                    guard,
                ) {
                    Ok(installed) => child = installed,
                    // 多个线程可能同时创建同一个子段。失败者丢弃自己未发布的段，
                    // 并沿用胜出线程已经安装的段。
                    Err(e) => child = e.current,
                }
            }

            debug_assert!(!child.is_null());
            segment = child;
            height -= 1;
        }

        // SAFETY: height == 1 说明当前段是叶子段，可以按 elements 解释 union。
        // 段一经发布，在整个 GrowableArray 生命周期内都不会被移除，所以该引用
        // 在 guard 所代表的访问期间保持有效。
        unsafe { &segment.deref().elements[index & SEGMENT_MASK] }
    }
}
