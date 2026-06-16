//! Thread-safe reference-counting pointers.
//! 线程安全的引用计数指针。
//!
//! See the [`Arc<T>`][Arc] documentation for more details.
//! 有关更多详情，请参阅 [`Arc<T>`][Arc] 文档。

use std::marker::PhantomData;
use std::ops::Deref;
use std::ptr::NonNull;
#[cfg(not(feature = "check-loom"))]
use std::sync::atomic::{AtomicUsize, Ordering, fence};
use std::{fmt, mem};

#[cfg(feature = "check-loom")]
use loom::sync::atomic::{AtomicUsize, Ordering, fence};

const MAX_REFCOUNT: usize = (isize::MAX) as usize;

/// A thread-safe reference-counting pointer. 'Arc' stands for 'Atomically
/// 一个线程安全的引用计数指针。“Arc”代表“原子地
/// Reference Counted'.
/// 引用计数。
///
/// The type `Arc<T>` provides shared ownership of a value of type `T`,
/// 类型 `Arc<T>` 提供对类型 `T` 值的共享所有权，
/// allocated in the heap. Invoking [`clone`][clone] on `Arc` produces
/// 分配在堆中。在 `Arc` 上调用 [`clone`][clone] 会产生
/// a new `Arc` instance, which points to the same allocation on the heap as the
/// 一个新的 `Arc` 实例，它指向与堆上的相同分配
/// source `Arc`, while increasing a reference count. When the last `Arc`
/// 源 `Arc`，同时增加引用计数。当最后一个 `Arc`
/// pointer to a given allocation is destroyed, the value stored in that allocation (often
/// 指向给定分配的指针被销毁，该分配中存储的值（通常
/// referred to as "inner value") is also dropped.
/// 所谓“内在价值”的部分也被放弃了。
///
/// Shared references in Rust disallow mutation by default, and `Arc` is no
/// Rust 中的共享引用默认不允许修改，并且 `Arc` 不是
/// exception: you cannot generally obtain a mutable reference to something
/// 异常：您通常不能获得某个对象的可变引用
/// inside an `Arc`. If you need to mutate through an `Arc`, use
/// 在一个 `Arc` 内部。如果你需要通过一个 `Arc` 进行变异，使用
/// [`Mutex`][mutex], [`RwLock`][rwlock], or one of the [`Atomic`][atomic]
/// [`Mutex`][互斥锁], [`RwLock`][读写锁], 或者其中一个 [`Atomic`][原子操作]
/// types.
/// 类型。
///
/// ## Thread Safety
/// ## 线程安全
///
/// Unlike [`Rc<T>`], `Arc<T>` uses atomic operations for its reference
/// 与 [`Rc<T>`] 不同，`Arc<T>` 对其引用使用原子操作
/// counting. This means that it is thread-safe. The disadvantage is that
/// 计数。这意味着它是线程安全的。缺点是
/// atomic operations are more expensive than ordinary memory accesses. If you
/// 原子操作比普通内存访问更耗费资源。如果你
/// are not sharing reference-counted allocations between threads, consider using
/// 不在线程之间共享引用计数分配时，可以考虑使用
/// [`Rc<T>`] for lower overhead. [`Rc<T>`] is a safe default, because the
/// [`Rc<T>`] 用于降低开销。[`Rc<T>`] 是一个安全的默认值，因为
/// compiler will catch any attempt to send an [`Rc<T>`] between threads.
/// 编译器会捕捉到任何尝试在线程之间发送 [`Rc<T>`] 的行为。
/// However, a library might choose `Arc<T>` in order to give library consumers
/// 然而，图书馆可能选择 `Arc<T>`，以便为图书馆使用者提供
/// more flexibility.
/// 更多灵活性。
///
/// `Arc<T>` will implement [`Send`] and [`Sync`] as long as the `T` implements
/// `Arc<T>` 将实施 [`Send`] 和 [`Sync`]，只要 `T` 实施
/// [`Send`] and [`Sync`]. Why can't you put a non-thread-safe type `T` in an
/// [`Send`] 和 [`Sync`]。为什么你不能在一个 `T` 中放入一个非线程安全类型
/// `Arc<T>` to make it thread-safe? This may be a bit counter-intuitive at
/// `Arc<T>` 要使其线程安全？这在某种程度上可能有点违反直觉
/// first: after all, isn't the point of `Arc<T>` thread safety? The key is
/// 首先：毕竟，`Arc<T>` 的重点不是线程安全吗？关键是
/// this: `Arc<T>` makes it thread safe to have multiple ownership of the same
/// 这个：`Arc<T>` 使得拥有多个相同的所有权线程安全
/// data, but it  doesn't add thread safety to its data. Consider
/// 数据，但它并没有为其数据增加线程安全性。考虑
/// <code>Arc<[RefCell\<T>]></code>. [`RefCell<T>`] isn't [`Sync`], and if `Arc<T>` was always
/// <code>Arc<[RefCell\<T>]></code>。[`RefCell<T>`]不是[`Sync`]，如果`Arc<T>`一直都是
/// [`Send`], <code>Arc<[RefCell\<T>]></code> would be as well. But then we'd have a problem:
/// [`Send`]、<code>Arc<[RefCell\<T>]></code>也会是。但那样我们就有个问题了：
/// [`RefCell<T>`] is not thread safe; it keeps track of the borrowing count using
/// [`RefCell<T>`] 不是线程安全的；它使用来跟踪借用计数
/// non-atomic operations.
/// 非原子操作。
///
/// In the end, this means that you may need to pair `Arc<T>` with some sort of
/// 最终，这意味着你可能需要将 `Arc<T>` 与某种类型的东西配对
/// [`std::sync`] type, usually [`Mutex<T>`][mutex].
/// [`std::sync`] 类型，通常为 [`Mutex<T>`][mutex]。
///
/// # Cloning references
/// # 克隆引用
///
/// Creating a new reference from an existing reference-counted pointer is done using the
/// 从现有的引用计数指针创建新引用是使用
/// `Clone` trait implemented for [`Arc<T>`][Arc].
/// 为 [`Arc<T>`][Arc] 实现了 `Clone` 特性。
///
/// ```
/// use cs431_homework::Arc;
/// let foo = Arc::new(vec![1.0, 2.0, 3.0]);
/// // The two syntaxes below are equivalent.
/// let a = foo.clone();
/// let b = Arc::clone(&foo);
/// // a, b, and foo are all Arcs that point to the same memory location
/// ```
///
/// ## `Deref` behavior
/// ## `Deref` 行为
///
/// `Arc<T>` automatically dereferences to `T` (via the [`Deref`] trait),
/// `Arc<T>` 会自动解引用到 `T`（通过 [`Deref`] 特性）,
/// so you can call `T`'s methods on a value of type `Arc<T>`. To avoid name
/// 这样你就可以在类型为 `Arc<T>` 的值上调用 `T` 的方法。为了避免名称
/// clashes with `T`'s methods, the methods of `Arc<T>` itself are associated
/// 与 `T` 的方法发生冲突，`Arc<T>` 本身的方法相关联
/// functions, called using [fully qualified syntax]:
/// 函数，通过[完全限定语法]调用：
///
/// ```
/// use cs431_homework::Arc;
///
/// let my_arc = Arc::new(5);
/// let my_five = Arc::try_unwrap(my_arc).unwrap();
/// ```
///
/// `Arc<T>`'s implementations of traits like `Clone` may also be called using
/// 像 `Clone` 这样的特征的 `Arc<T>` 实现也可以通过以下方式调用
/// fully qualified syntax. Some people prefer to use fully qualified syntax,
/// 完全限定的语法。有些人更喜欢使用完全限定的语法，
/// while others prefer using method-call syntax.
/// 而其他人则更喜欢使用方法调用语法。
///
/// ```
/// use cs431_homework::Arc;
///
/// let arc = Arc::new(());
/// // Method-call syntax
/// let arc2 = arc.clone();
/// // Fully qualified syntax
/// let arc3 = Arc::clone(&arc);
/// ```
///
/// [`Rc<T>`]: std::rc::Rc
/// [clone]: Clone::clone
/// [克隆]: Clone::clone
/// [mutex]: ../../std/sync/struct.Mutex.html
/// [互斥锁]: ../../std/sync/struct.Mutex.html
/// [rwlock]: ../../std/sync/struct.RwLock.html
/// [atomic]: core::sync::atomic
/// [RefCell\<T>]: core::cell::RefCell
/// [RefCell\<T>]：core::cell::RefCell
/// [`RefCell<T>`]: core::cell::RefCell
/// [`std::sync`]: ../../std/sync/index.html
/// [`Arc::clone(&from)`]: Arc::clone
/// [fully qualified syntax]: https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#fully-qualified-syntax-for-disambiguation-calling-methods-with-the-same-name
/// [完全限定语法]: https://doc.rust-lang.org/book/ch19-03-advanced-traits.html#fully-qualified-syntax-for-disambiguation-calling-methods-with-the-same-name
///
/// # Examples
/// # 示例
///
/// Sharing some immutable data between threads:
/// 在线程之间共享一些不可变数据：
///
/// ```no_run
/// use cs431_homework::Arc;
/// use std::thread;
///
/// let five = Arc::new(5);
///
/// for _ in 0..10 {
///     let five = Arc::clone(&five);
///
///     thread::spawn(move || {
///         println!("{five:?}");
///     });
/// }
/// ```
///
/// Sharing a mutable [`AtomicUsize`]:
/// 共享一个可变的 [`AtomicUsize`]：
///
/// [`AtomicUsize`]: core::sync::atomic::AtomicUsize "sync::atomic::AtomicUsize"
///
/// ```no_run
/// use cs431_homework::Arc;
/// use std::sync::atomic::{AtomicUsize, Ordering};
/// use std::thread;
///
/// let val = Arc::new(AtomicUsize::new(5));
///
/// for _ in 0..10 {
///     let val = Arc::clone(&val);
///
///     thread::spawn(move || {
///         let v = val.fetch_add(1, Ordering::SeqCst);
///         println!("{v:?}");
///     });
/// }
/// ```
///
/// See the [`rc` documentation][rc_examples] for more examples of reference
/// 有关引用的更多示例，请参见 [`rc` 文档][rc_examples]
/// counting in general.
/// 一般计数。
///
/// [rc_examples]: std::rc#examples
/// [rc_examples]: std::rc#例子
pub struct Arc<T> {
    ptr: NonNull<ArcInner<T>>,
    phantom: PhantomData<ArcInner<T>>,
}

unsafe impl<T: Sync + Send> Send for Arc<T> {}
unsafe impl<T: Sync + Send> Sync for Arc<T> {}

impl<T> Arc<T> {
    fn from_inner(ptr: NonNull<ArcInner<T>>) -> Self {
        Self {
            ptr,
            phantom: PhantomData,
        }
    }
}

struct ArcInner<T> {
    count: AtomicUsize,
    data: T,
}

unsafe impl<T: Sync + Send> Send for ArcInner<T> {}
unsafe impl<T: Sync + Send> Sync for ArcInner<T> {}

impl<T> Arc<T> {
    /// Constructs a new `Arc<T>`.
    /// 构造一个新的 `Arc<T>`。
    #[inline]
    pub fn new(data: T) -> Arc<T> {
        let x = Box::new(ArcInner {
            count: AtomicUsize::new(1),
            data,
        });
        Self::from_inner(Box::leak(x).into())
    }

    /// Returns a mutable reference into the given `Arc` if there are
    /// 如果存在，则返回对给定 `Arc` 的可变引用
    /// no other `Arc`. Otherwise, return `None`.
    /// 没有其他 `Arc`。否则，返回 `None`。
    ///
    /// # Examples
    /// # 示例
    ///
    /// ```
    /// use cs431_homework::Arc;
    ///
    /// let mut x = Arc::new(3);
    /// *Arc::get_mut(&mut x).unwrap() = 4;
    /// assert_eq!(*x, 4);
    ///
    /// let y = Arc::clone(&x);
    /// assert!(Arc::get_mut(&mut x).is_none());
    ///
    /// drop(y);
    /// assert!(Arc::get_mut(&mut x).is_some());
    /// ```
    #[inline]
    pub fn get_mut(this: &mut Self) -> Option<&mut T> {
        todo!()
    }

    // Used in `get_mut` and `make_mut` to check if the given `Arc` is the unique reference to the
    // 用于 `get_mut` 和 `make_mut` 中，以检查给定的 `Arc` 是否是唯一的引用
    // underlying data.
    // 基础数据。
    #[inline]
    fn is_unique(&mut self) -> bool {
        todo!()
    }

    /// Returns a mutable reference into the given `Arc` without any check.
    /// 返回对给定 `Arc` 的可变引用，而不进行任何检查。
    ///
    /// # Safety
    /// # 安全
    ///
    /// Any other `Arc` to the same allocation must not be dereferenced for the duration of the
    /// 在此期间，任何指向同一分配的其他 `Arc` 都不得被解引用
    /// returned borrow.  Specifically, call to this function must happen-after destruction of all
    /// returned 借用。具体来说，对此函数的调用必须发生在所有销毁之后
    /// the other `Arc` to the same allocation.
    /// 另一个 `Arc` 到相同的分配。
    ///
    /// # Examples
    /// # 示例
    ///
    /// ```
    /// use cs431_homework::Arc;
    ///
    /// let mut x = Arc::new(String::new());
    /// unsafe {
    ///     Arc::get_mut_unchecked(&mut x).push_str("foo")
    /// }
    /// assert_eq!(*x, "foo");
    /// ```
    pub unsafe fn get_mut_unchecked(this: &mut Self) -> &mut T {
        // We are careful to *not* create a reference covering the "count" fields, as
        // 我们小心不要创建一个覆盖“count”字段的引用，因为
        // this would alias with concurrent access to the reference counts (e.g. by `Weak`).
        // 这可能会与对引用计数的并发访问产生别名（例如，通过 `Weak`）。
        unsafe { &mut (*this.ptr.as_ptr()).data }
    }

    /// Gets the number of `Arc`s to this allocation. In addition, synchronize with the update that
    /// 获取此分配的 `Arc` 数量。此外，与更新同步
    /// this function reads from.
    /// 这个函数从……读取。
    ///
    /// # Safety
    /// # 安全
    ///
    /// This method by itself is safe, but using it correctly requires extra care.
    /// 这种方法本身是安全的，但正确使用它需要格外小心。
    /// Another thread can change the reference count at any time,
    /// 另一个线程可以随时更改引用计数，
    /// including potentially between calling this method and acting on the result.
    /// 包括可能在调用此方法与对结果进行操作之间的情况。
    ///
    /// # Examples
    /// # 示例
    ///
    /// ```
    /// use cs431_homework::Arc;
    ///
    /// let five = Arc::new(5);
    /// let _also_five = Arc::clone(&five);
    ///
    /// // This assertion is deterministic because we haven't shared
    /// // the `Arc` between threads.
    /// assert_eq!(2, Arc::count(&five));
    /// ```
    #[inline]
    pub fn count(this: &Self) -> usize {
        todo!()
    }

    #[inline]
    fn inner(&self) -> &ArcInner<T> {
        // This unsafety is ok because while this arc is alive we're guaranteed
        // 这种不安全是可以的，因为只要这个 Arc 存活，我们就是有保证的
        // that the inner pointer is valid. Furthermore, we know that the
        // 内部指针是有效的。此外，我们知道
        // `ArcInner` structure itself is `Sync` because the inner data is
        // `ArcInner`结构本身是`Sync`，因为内部数据是
        // `Sync` as well, so we're ok loaning out an immutable pointer to these
        // `Sync` 也是，所以我们可以放心把一个不可变指针借出给这些
        // contents.
        // 内容。
        unsafe { self.ptr.as_ref() }
    }

    /// Returns `true` if the two `Arc`s point to the same allocation
    /// 如果两个 `Arc` 指向同一分配，则返回 `true`
    /// (in a vein similar to `ptr::eq`).
    /// （以类似于 `ptr::eq` 的方式）。
    ///
    /// # Examples
    /// # 示例
    ///
    /// ```
    /// use cs431_homework::Arc;
    ///
    /// let five = Arc::new(5);
    /// let same_five = Arc::clone(&five);
    /// let other_five = Arc::new(5);
    ///
    /// assert!(Arc::ptr_eq(&five, &same_five));
    /// assert!(!Arc::ptr_eq(&five, &other_five));
    /// ```
    #[inline]
    pub fn ptr_eq(this: &Self, other: &Self) -> bool {
        this.ptr.as_ptr() == other.ptr.as_ptr()
    }

    /// Returns the inner value, if the given `Arc` is unique.
    /// 如果给定的 `Arc` 是唯一的，则返回内部值。
    ///
    /// Otherwise, an `Err` is returned with the same `Arc` that was passed in.
    /// 否则，将返回一个 `Err`，并带有传入的相同 `Arc`。
    ///
    /// # Examples
    /// # 示例
    ///
    /// ```
    /// use cs431_homework::Arc;
    ///
    /// let x = Arc::new(3);
    /// assert_eq!(Arc::try_unwrap(x).unwrap(), 3);
    ///
    /// let x = Arc::new(4);
    /// let _y = Arc::clone(&x);
    /// assert_eq!(*Arc::try_unwrap(x).unwrap_err(), 4);
    /// ```
    #[inline]
    pub fn try_unwrap(this: Self) -> Result<T, Self> {
        todo!()
    }
}

impl<T: Clone> Arc<T> {
    /// Makes a mutable reference into the given `Arc`.
    /// 将可变引用转换为指定的 `Arc`。
    ///
    /// If there are other `Arc` to the same allocation, then `make_mut` will create a new
    /// 如果有其他 `Arc` 指向相同的分配，那么 `make_mut` 将创建一个新的
    /// allocation and invoke `clone` on the inner value to ensure unique ownership. This is also
    /// 在内部值上分配并调用 `clone` 以确保唯一所有权。这也是
    /// referred to as clone-on-write.
    /// 称为写时克隆。
    ///
    /// See also `get_mut`, which will fail rather than cloning.
    /// 另请参见 `get_mut`，它会失败而不是克隆。
    ///
    /// # Examples
    /// # 示例
    ///
    /// ```
    /// use cs431_homework::Arc;
    ///
    /// let mut data = Arc::new(5);
    ///
    /// *Arc::make_mut(&mut data) += 1;         // Won't clone anything
    /// let mut other_data = Arc::clone(&data); // Won't clone inner data
    /// *Arc::make_mut(&mut data) += 1;         // Clones inner data
    /// *Arc::make_mut(&mut data) += 1;         // Won't clone anything
    /// *Arc::make_mut(&mut other_data) *= 2;   // Won't clone anything
    ///
    /// // Now `data` and `other_data` point to different allocations.
    /// assert_eq!(*data, 8);
    /// assert_eq!(*other_data, 12);
    /// ```
    #[inline]
    pub fn make_mut(this: &mut Self) -> &mut T {
        todo!()
    }
}

impl<T> Clone for Arc<T> {
    /// Makes a clone of the `Arc` pointer.
    /// 创建 `Arc` 指针的克隆。
    ///
    /// This creates another pointer to the same allocation, increasing the
    /// 这会创建另一个指向相同分配的指针，从而增加
    /// reference count.
    /// 引用计数。
    ///
    /// # Panics
    /// # panic
    ///
    /// This panics if the number of `Arc`s is larger than `isize::Max`.
    /// 如果 `Arc` 的数量大于 `isize::Max`，这会引起panic。
    ///
    /// # Examples
    /// # 示例
    ///
    /// ```
    /// use cs431_homework::Arc;
    ///
    /// let five = Arc::new(5);
    ///
    /// let _ = Arc::clone(&five);
    /// ```
    #[inline]
    fn clone(&self) -> Arc<T> {
        todo!()
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &T {
        &self.inner().data
    }
}

impl<T> Drop for Arc<T> {
    /// Drops the `Arc`.
    /// 丢下 `Arc`。
    ///
    /// This will decrement the reference count. If the reference
    /// 这将减少引用计数。如果引用
    /// count reaches zero, we `drop` the inner value.
    /// 当计数达到零时，我们 `drop` 内部值。
    ///
    /// # Examples
    /// # 示例
    ///
    /// ```
    /// use cs431_homework::Arc;
    ///
    /// struct Foo;
    ///
    /// impl Drop for Foo {
    ///     fn drop(&mut self) {
    ///         println!("dropped!");
    ///     }
    /// }
    ///
    /// let foo  = Arc::new(Foo);
    /// let foo2 = Arc::clone(&foo);
    ///
    /// drop(foo);    // Doesn't print anything
    /// drop(foo2);   // Prints "dropped!"
    /// ```
    fn drop(&mut self) {
        todo!()
    }
}

impl<T: fmt::Display> fmt::Display for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: fmt::Debug> fmt::Debug for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T> fmt::Pointer for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&(&**self), f)
    }
}
