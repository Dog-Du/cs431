# Core Arc
> 核心 Arc
**Implement a simplified version of `Arc` without support for `Weak`.**
**实现一个简化版本的 `Arc`，不支持 `Weak`。**

In this homework, you will practice release-acquire synchronization in weak memory
在这个作业中，你将练习在弱内存中进行release-acquire 同步
by implementing a simplified version of `Arc`.
通过实现 `Arc` 的简化版本。
Specifically, you will use atomic operations of `AtomicUsize`
具体来说，你将使用 `AtomicUsize` 的原子操作
to synchronize the accesses to the underlying data.
同步对底层数据的访问。

Fill in the `todo!()`s in `src/arc.rs`.
在 `src/arc.rs` 中填写 `todo!()`。
The total lines of code to be written is about 25.
要编写的代码总行数大约是25行。
The skeleton code is a heavily modified version of `Arc` from the standard library.
该框架代码是标准库中 `Arc` 的一个经过大量修改的版本。
We don't recommend reading the original source code before finishing this homework
我们不建议在完成这份作业之前阅读原始源码
because that version is more complex.
因为那个版本更复杂。

## ***2024 spring semester notice: Use `SeqCst`***
> ***2024年春季学期通知：使用`SeqCst`***
We won't cover the weak memory semantics in this semester.
本学期我们不会讲弱内存语义。
So you may ignore the instructions on `Ordering` stuff below and
所以你可以忽略下面关于 `Ordering` 的指示并
use `Ordering::SeqCst` for `ordering: Ordering` parameters for `std::sync::atomic` functions.
对于 `std::sync::atomic` 函数，使用 `ordering: Ordering` 参数的 `Ordering::SeqCst`。

## Guide
> 指南

Follow [the Arc section of the Rustnomicon (the book on unsafe Rust)][nomicon-arc].
请参考 [Rustnomicon 的 Arc 部分（关于不安全 Rust 的书）][nomicon-arc]。

Some food for thought on Rustnomicon's description:
关于 Rustnomicon 描述的一些思考:
* Quiz: Why does `Arc<T> : Sync` require `T : Send`?
  测验：为什么 `Arc<T> : Sync` 需要 `T : Send`？
* The [Layout section](https://doc.rust-lang.org/nomicon/arc-mutex/arc-layout.html) explains
  [Layout 一节](https://doc.rust-lang.org/nomicon/arc-mutex/arc-layout.html) 解释
  why [`NonNull`](https://doc.rust-lang.org/std/ptr/struct.NonNull.html) and `PhantomData` are necessary.
  为什么 [`NonNull`](https://doc.rust-lang.org/std/ptr/struct.NonNull.html) 和 `PhantomData` 是必要的。
  We don't care about them in this course and will not ask about them in the exams
  在这门课程中，我们不关心他们，也不会在考试中问到他们
  (it's quite interesting, though).
  （不过，这相当有趣）。
* Their implementation uses `fence(Acquire)`, which we may not be able cover in the lecture due to time constraints.
  他们的实现使用了 `fence(Acquire)`，由于时间限制，我们可能无法在讲座中涵盖它。
  You can implement (a slightly inefficient version of) Arc only with `AtomicUsize`'s methods and the concepts we covered in the lecture
  你可以仅使用 `AtomicUsize` 的方法和我们在讲座中讨论的概念来实现（一个稍微低效的版本的）Arc
  (you will need to use `Ordering::AcqRel` in some places).
  (你需要在某些地方使用 `Ordering::AcqRel`)。
  Using `fence(Acquire)` is not required in the homework and exam.
  在作业和考试中不要求使用 `fence(Acquire)`。
  If you want to fully understand the `fence(Acquire)` version,
  如果你想完全理解 `fence(Acquire)` 版本，
  read §4 of the [Promising semantics paper](https://sf.snu.ac.kr/publications/promising.pdf).
  阅读 [Promising semantics paper](https://sf.snu.ac.kr/publications/promising.pdf) 的第 §4 节。


### Synchronization requirements of `Arc`
> `Arc` 的同步要求

To ensure that data race does not occur in the implementation of Arc and the clients of Arc,
为了确保在 Arc 的实现以及 Arc 的客户端中不会发生数据竞争，
add enough synchronization operations to the Arc implementation:
向 Arc 实现中添加足够的同步操作：
* The initialization of the `ArcInner` memory block (in `new()`) happens before the accesses of its fields.
  `ArcInner` 内存块（在 `new()` 中）的初始化发生在其字段访问之前。
  (Guaranteeing this doesn't require extra synchronization operation in Arc. Quiz: Why?)
  （保证这一点在 Arc 中不需要额外的同步操作。小测验：为什么？）
* Accesses to the fields of `ArcInner` happen before the deallocation of the `ArcInner` memory block (in the last `drop()`).
  在释放 `ArcInner` 内存块之前（在最后的 `drop()` 中），会访问 `ArcInner` 的字段。
* Non-atomic writes to the data (via `&mut T` from `get_mut()`, `make_mut()`, `try_unwrap()`) happen after/before all the other accesses (via `&T` from `deref()`).
  对数据的非原子写入（通过 `&mut T` 从 `get_mut()`、`make_mut()`、`try_unwrap()`）发生在所有其他访问（通过 `&T` 从 `deref()`）之后/之前。
  More strictly, `&mut T` to the data must not concurrently coexist with `&T` (Rust's aliasing rule).
  更严格地说，`&mut T` 对数据的访问不得与 `&T` 同时共存（Rust 的别名规则）。


<!-- ## Grading (50 points) -->
<!-- ## 评分（50 分） -->
## Grading (40 points)
> 评分（40分）
Run `./scripts/grade-arc.sh`.
运行 `./scripts/grade-arc.sh`。

1. Functionality (25):
   功能性（25）：
   First, the grader will check if
   首先，评分脚本将检查是否
   your implementation passes the doc tests and the tests in `tests/arc.rs`.
   你的实现通过了文档测试和 `tests/arc.rs` 中的测试。
   You can manually re-run the test with the following commands:
   您可以使用以下命令手动重新运行测试：
    ```
    cargo test --test arc
    cargo test --doc arc
    source scripts/grade-utils.sh
    cargo_asan test --test arc
    cargo_asan test --doc arc
    ```
<!-- 1. Correctness (25): -->
<!-- 1. 正确性 (25)： -->
1. Correctness (15):
   正确性（15）：
   Then the grader runs the tests with
   然后评分脚本运行测试时
   [the Loom model checker](https://github.com/tokio-rs/loom)
   [Loom 模型检查器](https://github.com/tokio-rs/loom)
   to check all possible executions (interleaving & reordering) in the memory model.
   检查内存模型中所有可能的执行（交错和内存序）。
   <!--
   If your code doesn't pass these tests,
   如果你的代码没有通过这些测试，
   then you need to add more synchronization operations or
   那么你需要添加更多的同步操作或者
   fix the memory ordering of them.
   修复它们的内存序。
   -->
   You can manually re-run the tests with this command.
   你可以使用此命令手动重新运行测试。
    ```
    cargo test --features check-loom --test arc -- --nocapture --test-threads 1
    ```
<!--
1. Efficiency:
   效率：
   Make sure that you don't use `SeqCst` ordering.
   确保你不要使用 `SeqCst` 内存序。
   No points will be given if your solution contains `SeqCst`.
   如果你的解决方案包含 `SeqCst`，将不会获得任何分数。
   We will not check if your implementation is optimal in terms of synchronization,
   我们不会检查你的实现是否在同步方面是最优的，
   but we encourage you to find the minimal set of synchronization operations.
   但我们鼓励您找到最小的一组同步操作。
-->


## Submission
> 提交
Submit `arc.rs` to gg.
将 `arc.rs` 提交给 gg。


## Other Tips
> 其他提示
* Read
  读
  [`sync::atomic::AtomicUsize`](https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html) and
  [`sync::atomic::AtomicUsize`](https://doc.rust-lang.org/std/sync/atomic/struct.AtomicUsize.html) 和
  [`sync::Ordering`](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html).
  [`sync::Ordering`](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html)
  The semantics covered in the lectures applies to these.
  讲座中涉及的语义适用于这些。
* You may need to use
  你可能需要使用
  [`std::mem::forget`](https://doc.rust-lang.org/std/mem/fn.forget.html)
  [`std::mem::forget`](https://doc.rust-lang.org/std/mem/fn.forget.html)
  in `try_unwrap`.
  在 `try_unwrap`。
* If the test failure message is not descriptive enough,
  如果测试失败信息不够详细，
  try adding `-- --nocapture --test-threads 1`.
  试着添加 `-- --nocapture --test-threads 1`。

### FAQ: AddressSanitizer reports a memory leak in my implementation.
> 常见问题解答：AddressSanitizer 报告我的实现中存在内存泄漏。
It might be the case that
可能是这样
you're not deallocating the heap memory block in your `Drop` implementation.
你在你的 `Drop` 实现中没有释放堆内存块。
For example, if you call functions like `drop_in_place` on `*mut ArcInner<_>`,
例如，如果你在 `*mut ArcInner<_>` 上调用像 `drop_in_place` 这样的函数，
it only runs the destructor of `ArcInner`
它只运行 `ArcInner` 的析构函数
without freeing the memory where that `ArcInner` lived.
而不释放 `ArcInner` 所在的内存。

The standard method to free the heap memory block is to convert the pointer
释放堆内存块的标准方法是转换指针
`*mut T` to `Box<T>` whose destructor runs the destructor of `T` and frees the
`*mut T` 到 `Box<T>`，其析构函数运行 `T` 的析构函数并释放
heap memory occupied by `T`.
`T` 占用的堆内存。
For example, `pop_front_node` from HW2 uses `Box::from_raw` to convert the head
例如，HW2 中的 `pop_front_node` 使用 `Box::from_raw` 来转换头部
pointer into `Box<Node<_>>` and dereferences it,
指向 `Box<Node<_>>` 的指针并解引用它，
moving it out of the heap to a temporary location and freeing the memory block in the heap.
将其从堆中移到临时位置，并释放堆中的内存块。

For more information, see
欲了解更多信息，请参阅
<https://github.com/kaist-cp/cs431/issues/125>,
<https://doc.rust-lang.org/reference/destructors.html>, and
<https://doc.rust-lang.org/reference/destructors.html>，并且
<https://doc.rust-lang.org/std/boxed/index.html>.


[nomicon-arc]: https://doc.rust-lang.org/nomicon/arc-mutex/arc.html
`[nomicon-arc]` 引用 Rustonomicon 中的 Arc 章节。
[ORC11]: https://plv.mpi-sws.org/rustbelt/rbrlx/
