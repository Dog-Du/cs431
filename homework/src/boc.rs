//! Concurrent Owner (Cown) type.
//! 并发所有者（Cown）类型。

use core::cell::UnsafeCell;
use core::sync::atomic::Ordering::SeqCst;
use core::sync::atomic::{AtomicBool, AtomicPtr, AtomicUsize};
use core::{fmt, hint, ptr};
use std::sync::Arc;

/// A trait representing a `Cown`.
/// 表示 `Cown` 的特性。
///
/// Instead of directly using a `Cown<T>`, which fixes _a single_ `T` we use a trait object to allow
/// 我们不直接使用一个 `Cown<T>`，它固定了 _一个_ `T`，而是使用一个特征对象以允许
/// multiple requests with different `T`s to be used with the same cown.
/// 使用相同 cown 的多个请求，每个请求使用不同的 `T`。
///
/// # Safety
/// # 安全
///
/// `last` should actually return the last request for the corresponding cown.
/// `last` 实际上应该返回对应牛的最后一次请求。
unsafe trait CownBase: Send {
    /// Return a pointer to the tail of this cown's request queue.
    /// 返回指向此 cown 的请求队列尾部的指针。
    fn last(&self) -> &AtomicPtr<Request>;
}

/// A request for a cown.
/// 一份关于奶牛的请求。
pub struct Request {
    /// Pointer to the next scheduled behavior.
    /// 指向下一个预定行为的指针。
    next: AtomicPtr<Behavior>,
    /// Is this request scheduled?
    /// 这个请求安排好了吗？
    scheduled: AtomicBool,
    /// The cown that this request wants to access.
    /// 此请求想要访问的农场的牛。
    ///
    /// This is an `Arc` as the all exposed `CownPtr`s may have been dropped while the behavior is
    /// 这是一个 `Arc`，因为所有暴露的 `CownPtr` 可能已经被丢弃，而行为是
    /// still scheduled.
    /// 仍然如期安排。
    target: Arc<dyn CownBase>,
}

// SAFETY: In the basic version of BoC, user cannot get shared reference through the [`CownBase`],
// 安全性：在 BoC 的基本版本中，用户无法通过 [`CownBase`] 获取共享引用，
// so `Sync` bound on it is not necessary.
// 所以对它的 `Sync` 绑定不是必要的。
unsafe impl Send for Request {}

impl Request {
    /// Creates a new Request.
    /// 创建一个新的请求。
    fn new(target: Arc<dyn CownBase>) -> Request {
        Request {
            next: AtomicPtr::new(ptr::null_mut()),
            scheduled: AtomicBool::new(false),
            target,
        }
    }

    /// Start the first phase of the 2PL enqueue operation.
    /// 开始 2PL 入队操作的第一阶段。
    ///
    /// Enqueues `self` onto the `target` cown. Returns once all previous behaviors on this cown has
    /// 将 `self` 排入 `target` cown。 当此 cown 上的所有先前行为完成后返回
    /// finished enqueueing on all of its required cowns. This ensures the 2PL protocol.
    /// 已在其所有必需的 cown 上完成入队。这确保了 2PL 协议。
    ///
    /// # SAFETY
    /// # 安全
    ///
    /// `behavior` must be a valid raw pointer to the behavior for `self`, and this should be the
    /// `behavior` 必须是 `self` 行为的有效裸指针，并且这应该是
    /// only enqueueing of this request and behavior.
    /// 仅对该请求和行为进行排队。
    unsafe fn start_enqueue(&self, behavior: *const Behavior) {
        todo!()
    }

    /// Finish the second phase of the 2PL enqueue operation.
    /// 完成 2PL 入队操作的第二阶段。
    ///
    /// Sets the scheduled flag so that subsequent behaviors can continue the 2PL enqueue.
    /// 设置计划标志，以便后续行为可以继续 2PL 排队。
    ///
    /// # Safety
    /// # 安全
    ///
    /// All enqueues for smaller requests on this cown must have been completed.
    /// 此 cown 上较小请求的所有入队操作必须已完成。
    unsafe fn finish_enqueue(&self) {
        todo!()
    }

    /// Release the cown to the next behavior.
    /// 将奶牛释放到下一个行为。
    ///
    /// Called when `self` has been completed, and thus can allow the next waiting behavior to run.
    /// 在 `self` 完成时调用，从而允许下一个等待的行为运行。
    /// If there is no next behavior, then the cown's tail pointer is set to null.
    /// 如果没有下一个行为，那么牛的尾指针将被设置为 null。
    ///
    /// # Safety
    /// # 安全
    ///
    /// `self` must have been actually completed.
    /// `self` 必须已经实际完成。
    unsafe fn release(&self) {
        todo!()
    }
}

impl Ord for Request {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        #[allow(warnings)]
        Arc::as_ptr(&self.target).cmp(&Arc::as_ptr(&other.target))
    }
}
impl PartialOrd for Request {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for Request {
    fn eq(&self, other: &Self) -> bool {
        matches!(self.cmp(other), core::cmp::Ordering::Equal)
    }
}
impl Eq for Request {}

impl fmt::Debug for Request {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Request")
            .field("next", &self.next)
            .field("scheduled", &self.scheduled)
            .finish()
    }
}

/// The value should only be accessed inside a `when!` block.
/// 该值应仅在 `when!` 块内访问。
#[derive(Debug)]
struct Cown<T: Send> {
    /// MCS lock tail.
    /// MCS锁尾。
    ///
    /// When a new node is enqueued, the enqueuer of the previous tail node will wait until the
    /// 当一个新节点被加入队列时，前一个尾节点的加入者将等待直到
    /// current enqueuer sets that node's `.next`.
    /// 当前入队者设置该节点的 `.next`。
    last: AtomicPtr<Request>,
    /// The value of this cown.
    /// 这头奶牛的价值。
    value: UnsafeCell<T>,
}

// SAFETY: `self.tail` is indeed the actual tail.
// 安全：`self.tail` 确实是真正的尾部。
unsafe impl<T: Send> CownBase for Cown<T> {
    fn last(&self) -> &AtomicPtr<Request> {
        &self.last
    }
}

/// Public interface to Cown.
/// Cown 的公共接口。
#[derive(Debug)]
pub struct CownPtr<T: Send> {
    inner: Arc<Cown<T>>,
}

// SAFETY: In the basic version of BoC, user cannot get `&T`, so `Sync` is not necessary.
// 安全：在BoC的基本版本中，用户无法获得`&T`，因此`Sync`不是必需的。
unsafe impl<T: Send> Send for CownPtr<T> {}

impl<T: Send> Clone for CownPtr<T> {
    fn clone(&self) -> Self {
        CownPtr {
            inner: self.inner.clone(),
        }
    }
}

impl<T: Send> CownPtr<T> {
    /// Creates a new Cown.
    /// 创建一个新的 Cown。
    pub fn new(value: T) -> CownPtr<T> {
        CownPtr {
            inner: Arc::new(Cown {
                last: AtomicPtr::new(ptr::null_mut()),
                value: UnsafeCell::new(value),
            }),
        }
    }
}

type BehaviorThunk = Box<dyn FnOnce() + Send>;

/// Behavior that captures the content of a when body.
/// 捕捉身体内容的行为。
struct Behavior {
    /// The body of the Behavior.
    /// 行为的主体。
    thunk: BehaviorThunk,
    /// Number of not-yet enqueued requests.
    /// 尚未排队的请求数量。
    count: AtomicUsize,
    /// The requests for this behavior.
    /// 对这种行为的要求。
    requests: Vec<Request>,
}

impl Behavior {
    /// Schedules the Behavior.
    /// 安排行为。
    ///
    /// Performs two phase locking (2PL) over the enqueuing of the requests.
    /// 对请求的入队执行两阶段锁（2PL）。
    /// This ensures that the overall effect of the enqueue is atomic.
    /// 这确保了入队操作的整体效果是原子的。
    fn schedule(self) {
        todo!()
    }

    /// Resolves a single outstanding request for `this`.
    /// 解决 `this` 的单个未完成请求。
    ///
    /// Called when a request for `this` is at the head of the queue for a particular cown. If it is
    /// 当某个特定牛的队列头部有 `this` 请求时调用。如果是
    /// the last request, then the thunk is scheduled.
    /// 最后的请求，然后 thunk 被安排。
    ///
    /// # Safety
    /// # 安全
    ///
    /// `this` must be a valid behavior.
    /// `this` 必须是一种有效的行为。
    unsafe fn resolve_one(this: *const Self) {
        todo!()
    }
}

impl fmt::Debug for Behavior {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Behavior")
            .field("thunk", &"BehaviorThunk")
            .field("count", &self.count)
            .field("requests", &self.requests)
            .finish()
    }
}

// TODO: terminator?
// 待办事项：终结者？
impl Behavior {
    fn new<C, F>(cowns: C, f: F) -> Behavior
    where
        C: CownPtrs + Send + 'static,
        F: for<'l> Fn(C::CownRefs<'l>) + Send + 'static,
    {
        todo!()
    }
}

/// Trait for a collection of `CownPtr`s.
/// `CownPtr`集合的特征。
///
/// Users pass `CownPtrs` to `when!` clause to specify a collection of shared resources, and such
/// 用户将 `CownPtrs` 传递给 `when!` 条款以指定一组共享资源，以及此类
/// resources can be accessed via `CownRefs` inside the thunk.
/// 资源可以通过 thunk 内的 `CownRefs` 访问。
///
/// # Safety
/// # 安全
///
/// `requests` should actually return the requests for the corresponding cowns.
/// `requests` 实际上应该返回对应 cowns 的请求。
pub unsafe trait CownPtrs {
    /// Types for references corresponding to `CownPtrs`.
    /// 与 `CownPtrs` 对应的引用类型。
    type CownRefs<'l>
    where
        Self: 'l;

    /// Returns a collection of `Request`.
    /// 返回 `Request` 的集合。
    // This could return a `Box<[Request]>`, but we use a `Vec` to avoid possible reallocation in
    // 这可能会返回一个 `Box<[Request]>`，但我们使用 `Vec` 来避免可能的重新分配
    // the implementation.
    // 实施。
    fn requests(&self) -> Vec<Request>;

    /// Returns mutable references of type `CownRefs`.
    /// 返回类型为 `CownRefs` 的可变引用。
    ///
    /// # Safety
    /// # 安全
    ///
    /// Must be called only if it is safe to access the shared resources.
    /// 只有在安全访问共享资源时才应调用。
    unsafe fn get_mut<'l>(self) -> Self::CownRefs<'l>;
}

unsafe impl CownPtrs for () {
    type CownRefs<'l>
        = ()
    where
        Self: 'l;

    fn requests(&self) -> Vec<Request> {
        Vec::new()
    }

    unsafe fn get_mut<'l>(self) -> Self::CownRefs<'l> {}
}

unsafe impl<T: Send + 'static, Ts: CownPtrs> CownPtrs for (CownPtr<T>, Ts) {
    type CownRefs<'l>
        = (&'l mut T, Ts::CownRefs<'l>)
    where
        Self: 'l;

    fn requests(&self) -> Vec<Request> {
        let mut rs = self.1.requests();
        let cown_base: Arc<dyn CownBase> = self.0.inner.clone();
        rs.push(Request::new(cown_base));
        rs
    }

    unsafe fn get_mut<'l>(self) -> Self::CownRefs<'l> {
        unsafe { (&mut *self.0.inner.value.get(), self.1.get_mut()) }
    }
}

unsafe impl<T: Send + 'static> CownPtrs for Vec<CownPtr<T>> {
    type CownRefs<'l>
        = Vec<&'l mut T>
    where
        Self: 'l;

    fn requests(&self) -> Vec<Request> {
        self.iter().map(|x| Request::new(x.inner.clone())).collect()
    }

    unsafe fn get_mut<'l>(self) -> Self::CownRefs<'l> {
        self.iter()
            .map(|x| unsafe { &mut *x.inner.value.get() })
            .collect()
    }
}

/// Creates a `Behavior` and schedules it. Used by "When" block.
/// 创建一个 `Behavior` 并安排它的时间。由 “When” 块使用。
pub fn run_when<C, F>(cowns: C, f: F)
where
    C: CownPtrs + Send + 'static,
    F: for<'l> Fn(C::CownRefs<'l>) + Send + 'static,
{
    Behavior::new(cowns, f).schedule();
}

/// from <https://docs.rs/tuple_list/latest/tuple_list/>
/// 来自 <https://docs.rs/tuple_list/latest/tuple_list/>
#[macro_export]
macro_rules! tuple_list {
    () => ( () );

    // handling simple identifiers, for limited types and patterns support
    // 处理简单标识符，支持有限的类型和模式
    ($i:ident)  => ( ($i, ()) );
    ($i:ident,) => ( ($i, ()) );
    ($i:ident, $($e:ident),*)  => ( ($i, $crate::tuple_list!($($e),*)) );
    ($i:ident, $($e:ident),*,) => ( ($i, $crate::tuple_list!($($e),*)) );

    // handling complex expressions
    // 处理复杂表达式
    ($i:expr_2021)  => ( ($i, ()) );
    ($i:expr_2021,) => ( ($i, ()) );
    ($i:expr_2021, $($e:expr_2021),*)  => ( ($i, $crate::tuple_list!($($e),*)) );
    ($i:expr_2021, $($e:expr_2021),*,) => ( ($i, $crate::tuple_list!($($e),*)) );
}

/// "When" block.
/// “When” 块。
#[macro_export]
macro_rules! when {
    ( $( $cs:ident ),* ; $( $gs:ident ),* ; $thunk:expr_2021 ) => {{
        run_when(tuple_list!($($cs.clone()),*), move |tuple_list!($($gs),*)| $thunk);
    }};
}

#[test]
fn boc() {
    let c1 = CownPtr::new(0);
    let c2 = CownPtr::new(0);
    let c3 = CownPtr::new(false);
    let c2_ = c2.clone();
    let c3_ = c3.clone();

    let (finish_sender, finish_receiver) = crossbeam_channel::bounded(0);

    when!(c1, c2; g1, g2; {
        // c3, c2 are moved into this thunk. There's no such thing as auto-cloning move closure.
        // c3、c2 被移入了这个 thunk。没有自动克隆移动闭包这样的东西。
        *g1 += 1;
        *g2 += 1;
        when!(c3, c2; g3, g2; {
            *g2 += 1;
            *g3 = true;
        });
    });

    when!(c1, c2_, c3_; g1, g2, g3; {
        assert_eq!(*g1, 1);
        assert_eq!(*g2, if *g3 { 2 } else { 1 });
        finish_sender.send(()).unwrap();
    });

    // wait for termination
    // 等待终止
    finish_receiver.recv().unwrap();
}

#[test]
fn boc_vec() {
    let c1 = CownPtr::new(0);
    let c2 = CownPtr::new(0);
    let c3 = CownPtr::new(false);
    let c2_ = c2.clone();
    let c3_ = c3.clone();

    let (finish_sender, finish_receiver) = crossbeam_channel::bounded(0);

    run_when(vec![c1.clone(), c2.clone()], move |mut x| {
        // c3, c2 are moved into this thunk. There's no such thing as auto-cloning move closure.
        // c3、c2 被移入了这个 thunk。没有自动克隆移动闭包这样的东西。
        *x[0] += 1;
        *x[1] += 1;
        when!(c3, c2; g3, g2; {
            *g2 += 1;
            *g3 = true;
        });
    });

    when!(c1, c2_, c3_; g1, g2, g3; {
        assert_eq!(*g1, 1);
        assert_eq!(*g2, if *g3 { 2 } else { 1 });
        finish_sender.send(()).unwrap();
    });

    // wait for termination
    // 等待终止
    finish_receiver.recv().unwrap();
}
