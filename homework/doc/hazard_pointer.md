# Hazard pointers
> Hazard Pointer
**Implement Hazard Pointers.**
**实现Hazard Pointer。**

Fill in the `todo!()`s in
填写 `todo!()`s 于
[`hazard_pointer/hazard.rs`](../src/hazard_pointer/hazard.rs) and
[`hazard_pointer/hazard.rs`](../src/hazard_pointer/hazard.rs) 和
[`hazard_pointer/retire.rs`](../src/hazard_pointer/retire.rs)
[`hazard_pointer/retire.rs`](../src/hazard_pointer/retire.rs)
(approx 75 lines).
（约75行）。

This homework is in 2 parts:
这个作业分为两部分：
1. (70 points) Functionality:
   (70 分) 功能性：
   Implement hazard pointers with `Ordering::SeqCst` for every atomic access.
   对每次原子访问使用 `Ordering::SeqCst` 实现Hazard Pointer。
   With this, we can pretend that we are on a sequentially consistent memory model.
   有了这个，我们可以假装我们在一个顺序一致内存模型上。
2. (30 points) Performance:
   (30 分) 性能：
   After you learn about relaxed memory semantics,
   在你了解了松弛内存语义之后，
   optimize the implementation by relaxing the ordering.
   通过放宽内存序来优化实现。
   We recommend working on this part after finishing the [Arc homework](./arc.md).
   我们建议在完成 [Arc 作业](./arc.md) 之后再处理这一部分。

## ***2024 spring semester notice: Part 2 is cancelled***
> ***2024年春季学期通知：第二部分取消***
We won't cover the weak memory semantics in this semester.
本学期我们不会讲弱内存语义。
To ensure that the grader works properly, you must use `Ordering:SeqCst` for all operations.
为了确保评分脚本正常工作，您必须在所有操作中使用 `Ordering:SeqCst`。

## Part 1: Hazard pointers in the sequentially consistent memory model
> 第 1 部分：顺序一致内存模型中的 Hazard Pointer

Read [this paper](https://ieeexplore.ieee.org/document/1291819).
阅读 [this paper](https://ieeexplore.ieee.org/document/1291819)。
While this paper is sufficient for understanding Hazard Pointers,
虽然这篇论文足以理解Hazard Pointer，
you may also want to take a look at [WG21 P2530](https://wg21.link/p2530),
你也可能想看看 [WG21 P2530](https://wg21.link/p2530),
the proposal for adding Hazard Pointers to the C++ standard library.
向 C 标准库添加 Hazard Pointer 的提议。
The rest of this section summarizes the algorithm and correctness argument of hazard pointers.
本节的其余部分总结了Hazard Pointer的算法和正确性论证。


Suppose a data structure has a memory block b.
假设一个数据结构有一个内存块 b。
A thread (T1) wants to read the value written in b and
一个线程 (T1) 想要读取写入 b 的值并
another thread (T2) wants to remove b from the data structure and free the memory.
另一个线程 (T2) 想要从数据结构中移除 b 并释放内存。
To prevent use-after-free,
为了防止使用已释放的内存，
T1 has to ensure that b is not freed before reading b and
T1必须确保在读取b之前b不会被释放，并且
T2 has to check that no other threads are accessing b before freeing b.
T2 必须检查在释放 b 之前没有其他线程在访问 b。
The hazard pointer library implements this mechanism as follows:
Hazard Pointer 库通过以下方式实现此机制：

```
(T1-1) Add b to the hazard list       | (T2-1) Unlink b and `retire(b)`
       (`Shield::try_protect()`)      |
(T1-2) Check if b is still reachable  | (T2-2) Check if b is in the hazard list
       if so, deref b                 |        if not, free b
(T1-3) Remove b from the hazard list  |
       (`Shield::drop()`)             |
```

To show that the algorithm prevents use-after-free,
为了显示该算法可以防止使用后释放，
let's consider all possible interleavings of each step
让我们考虑每一步的所有可能交错方式
(in the sequentially consistent memory model).
(在顺序一致内存模型中)。

First, if `T1-3 → T2-2` (`T2-2` is executed after `T1-3`),
首先，如果 `T1-3 → T2-2`（`T2-2` 在 `T1-3` 之后执行），
then b is freed after all accesses.
那么 b 在所有访问之后被释放。

Second, in all the remaining cases,
其次，在所有其余情况下，
either `T1-1 → T2-2` or `T2-1 → T1-2` holds
`T1-1 → T2-2` 或 `T2-1 → T1-2` 成立
(otherwise, there is a cycle `T1-1 → T1-2 → T2-1 → T2-2 → T1-1`).
（否则，会有一个循环 `T1-1 → T1-2 → T2-1 → T2-2 → T1-1`）。
- If `T1-1 → T2-2`, then b is not freed.
  如果 `T1-1 → T2-2`，那么 b 不会被释放。
- If `T2-1 → T1-2`, then the validation fails, so `T1` will not dereference b.
  如果 `T2-1 → T1-2`，则验证失败，因此 `T1` 不会解除对 b 的引用。

Therefore, the algorithm is correct in the sequentially consistent memory model.
因此，该算法在顺序一致内存模型中是正确的。


## Part 2: Relaxing the orderings
> 第 2 部分：放宽内存序

If you use `Ordering::Relaxed`,
如果你使用 `Ordering::Relaxed`，
the correctness argument from the previous section doesn't hold.
前一节中的正确性论证不成立。
The problem is that in the relaxed memory model,
问题在于在 Relaxed 内存模型中，
`→` ("executed before") doesn't imply that
`→`（“在...之前执行”）并不意味着那样
the latter instruction sees the effect of the earlier instruction.
后一个指令会看到前一个指令的效果。
To fix this, we should add some synchronization operations
为了解决这个问题，我们应该添加一些同步操作
so that `→` implies "happens-before".
以便 `→` 意味着“先于发生”。

First, if `T2-2` saw the result of `T1-3`,
首先，如果 `T2-2` 看到了 `T1-3` 的结果，
then we want to enforce `deref b @ T1` happens before `free b @ T2`
然后我们希望强制 `deref b @ T1` 在 `free b @ T2` 之前发生
To enforce this,
为了执行这一点，
it suffices to add release-acquire synchronization between `T1-3` and `T2-2`
只需在 `T1-3` 和 `T2-2` 之间添加release-acquire 同步即可
(recall the synchronization in `Arc::drop`).
（回想 `Arc::drop` 中的同步。）

For the second case, release-acquire doesn't guarantee
对于第二种情况，release-acquire并不保证
"either `T1-1` happens before `T2-2` or `T2-1` happens before `T1-2`".
“要么 `T1-1` 在 `T2-2` 之前发生，要么 `T2-1` 在 `T1-2` 之前发生。”
Because of that, `T1-2` may not read the message of `T2-1`
因此，`T1-2` 可能无法读取 `T2-1` 的消息
and `T2-2` may not read the message of `T1-2` at the same time,
并且 `T2-2` 可能无法同时阅读 `T1-2` 的消息，
leading to concurrent `deref b` and `free b`.
导致同时发生 `deref b` 和 `free b`。
To make this work, we should insert an SC fence (`fence(SeqCst)`)
为了使这个工作，我们应该插入一个 SC 栅栏（`fence(SeqCst)`）
between `T1-1` and `T1-2`, and another between `T2-1` and `T2-2`.
在`T1-1`和`T1-2`之间，另一个在`T2-1`和`T2-2`之间。
<!-- This should be explained in the lecture.
<!-- 这应该在讲座中解释。 -->
Recall that an SC fence joins the executing thread's view and the global SC view.
请记住，顺序一致（SC）屏障连接了执行线程的视图和全局顺序一致视图。
This means that
这意味着
the view of a thread after executing its SC fence
线程在执行其 SC 栅栏后的视图
is entirely included in the view of another thread after its SC fence.
在另一个线程的 SC 栅栏之后，完全包含在其视图中。
If we insert an SC fence between
如果我们在之间插入一个SC围栏
`T1-1` and `T1-2`, and another between `T2-1` and `T2-2`,
`T1-1` 和 `T1-2`，以及另一个在 `T2-1` 和 `T2-2` 之间，
then either `T1's fence ⊑ T2's fence` or `T2's fence ⊑ T1's fence` holds.
那么要么 `T1's fence ⊑ T2's fence` 要么 `T2's fence ⊑ T1's fence` 成立。
Therefore, `T1-1 ⊑ T2-2` or `T2-1 ⊑ T1-2`.
因此，`T1-1 ⊑ T2-2` 或 `T2-1 ⊑ T1-2`。
-->

## Grading (100 points)
> 评分（100分）
Run `./scripts/grade-hazard_pointer.sh`.
运行 `./scripts/grade-hazard_pointer.sh`。

### Part 1: Functionality (70 points)
> 第 1 部分：功能正确性（70 分）
Like [hash table](./hash_table.md), we will first test if your implementation with `SeqCst` ordering is correct.
像 [hash table](./hash_table.md) 一样，我们将首先测试你的实现是否符合 `SeqCst` 内存序的正确性。
* tested with `cargo[_asan,_tsan] [--release]`
  使用 `cargo[_asan,_tsan] [--release]` 测试
    * tests in `hazard.rs` (20 points)
      `hazard.rs` 的测试（20 分）
    * a test in `retire.rs` (10 points)
      `retire.rs` 的一次测试（10 分）
    * tests in `tests/hazard_pointer.rs` (40 points)
      `tests/hazard_pointer.rs` 的测试（40 分）

### Part 2: Relaxed orderings (30 points)
> 第 2 部分：宽松内存序（30 分）
Like [arc](./arc.md), we will additionally use the loom model checker to test your hazard pointer implementation with relaxed orderings.
像 [arc](./arc.md) 一样，我们还将使用 loom 模型检查器来测试您使用 Relaxed 内存序的 Hazard Pointer 实现。
* tested with `cargo --features check-loom`
  使用 `cargo --features check-loom` 测试
    * tests in `tests/hazard_pointer.rs` `mod sync` (30 points)
      `tests/hazard_pointer.rs` `mod sync` 测试（30 分）

Note that we will also run the tests for part 1 as well,
请注意，我们也将运行第一部分的测试，
so make sure your implementation still passes all tests.
所以确保你的实现仍然能通过所有测试。

## Submission
> 提交
```bash
cd cs431/homework
./scripts/submit.sh
ls ./target/hw-hazard_pointer.zip
```
Submit `hw-hazard_pointer.zip` to gg.
将 `hw-hazard_pointer.zip` 提交给 gg。

## FAQ
> 常见问题

> loom throws an error when I used `get_mut()` on a `AtomicPtr`.
> 当我在 `AtomicPtr` 上使用 `get_mut()` 时，loom 抛出一个错误。

Currently, loom does not understand `get_mut()`
目前，Loom 不支持 `get_mut()`
(<https://github.com/tokio-rs/loom/issues/154>).
Please use `load()` with `Ordering::Relaxed` instead.
请改用 `Ordering::Relaxed` 替代 `load()`。
