# Concurrent set based on Lock-coupling linked list
> 基于锁耦合链表的并发集合
**Implement concurrent set data structures with sorted singly linked list using fine-grained lock-coupling.**
**使用细粒度锁耦合实现带排序单向链表的并发集合数据结构。**

Suppose you want a set data structure that supports concurrent operations.
假设你想要一个支持并发操作的集合数据结构。
The simplest possible approach would be taking a non-concurrent set implementation and protecting it with a global lock.
最简单的方法可能是采用非并发的集合实现，并用全局锁来保护它。
However, this is not a great idea if the set is accessed frequently because a thread's operation blocks all the other threads' operations.
然而，如果集合被频繁访问，这不是一个好主意，因为一个线程的操作会阻塞所有其他线程的操作。

In this homework, you will write an implementation of the set data structure based on singly linked list protected by fine-grained locks.
在这个作业中，你将编写一个基于单向链表并由细粒度锁保护的集合数据结构的实现。
* The nodes in the list are sorted by their value, so that one can efficiently check if a value is in the set.
  列表中的节点按它们的值排序，因此可以高效地检查某个值是否在集合中。
* Each node has its own lock that protects its `next` field.
  每个节点都有自己的锁来保护其 `next` 字段。
  When traversing the list, the locks are acquired and released in the hand-over-hand manner.
  在遍历列表时，锁以逐步传递的方式被获取和释放。
  This allows multiple operations run more concurrently.
  这允许多个操作更加并发地运行。

Fill in the `todo!()`s in `list_set/fine_grained.rs` (about 40 lines of code).
在 `list_set/fine_grained.rs` 中填写 `todo!()`（大约 40 行代码）。
As in the [Linked List homework](./linked_list.md), you will need to use some unsafe operations.
和在 [Linked List homework](./linked_list.md) 中一样，你将需要使用一些不安全的操作。

## Testing
> 测试
Tests are defined in `tests/list_set/fine_grained.rs`.
测试在 `tests/list_set/fine_grained.rs` 中定义。
Some of them use the common set test functions defined in `src/test/adt/set.rs`.
其中一些使用在 `src/test/adt/set.rs` 中定义的通用集合测试函数。

## Grading (45 points)
> 评分（45分）
Run
跑
```
./scripts/grade-list_set.sh
```

The grader runs the tests
评分脚本运行测试
with `cargo`, `cargo_asan`, and `cargo_tsan` in the following order.
按以下顺序使用 `cargo`、`cargo_asan` 和 `cargo_tsan`。
1. `stress_sequential` (5 points)
   `stress_sequential`（5 分）
1. `stress_concurrent` (10 points)
   `stress_concurrent`（10 分）
1. `log_concurrent` (15 points)
   `log_concurrent`（15 分）
1. `iter_consistent` (15 points)
   `iter_consistent`（15 分）

For the above tests, if a test fails in a module, then the later tests in the same module will not be run.
对于上述测试，如果某个模块中的测试失败，则该模块中后续的测试将不会运行。

## Submission
> 提交
```sh
cd cs431/homework
./scripts/submit.sh
ls ./target/hw-list_set.zip
```

Submit `hw-list_set.zip` to gg.
将 `hw-list_set.zip` 提交给 gg。

## Advanced (optional)
> 进阶（可选）
**Note**: This is an *optional* homework, meaning that it will not be graded and not be asked in the exam.
**注意**：这是一个*可选*作业，这意味着它不会被评分，也不会在考试中出现。

Consider a variant of the homework that uses `SeqLock` instead of `Mutex`.
考虑一个作业的变体，它使用 `SeqLock` 而不是 `Mutex`。
This allows read operations to run optimistically without actually locking.
这允许读操作乐观地运行而无需实际加锁。
Therefore, read operations are more efficient in read-most scenario, and
因此，在以读取为主的场景中，读取操作更高效，并且
they do not block other operations.
它们不会阻塞其他操作。
However, more care must be taken to ensure correctness.
然而，必须更加小心以确保正确性。
  * You need to validate read operations and handle the failure.
    你需要验证读取操作并处理失败情况。
      * Do not use `ReadGuard::restart()`.
        不要使用 `ReadGuard::restart()`。
        Using this correctly requires some extra synchronization
        正确使用这个需要一些额外的同步
        (to be covered in lock-free list lecture),
        （将在无锁列表讲座中讲解），
        which makes `SeqLock` somewhat pointless.
        这使得 `SeqLock` 有些毫无意义。
        The tests assume that `ReadGuard::restart()` is not used.
        测试假设 `ReadGuard::restart()` 没有被使用。
  * Since each node can be read and modified to concurrently,
    由于每个节点可以被同时读取和修改，
    you should use atomic operations to avoid data races.
    你应该使用原子操作以避免数据竞争。
    Specifically, you will use `crossbeam_epoch`'s `Atomic<T>` type
    具体来说，您将使用 `crossbeam_epoch` 的 `Atomic<T>` 类型
    (instead of `std::sync::AtomicPtr<T>`, due to the next issue).
    （由于下一个问题，而不是 `std::sync::AtomicPtr<T>`）。
    For `Ordering`, use `SeqCst` everywhere.
    对于 `Ordering`，在所有地方使用 `SeqCst`。
    (In the later part of this course, you will learn that `Relaxed` is sufficient.
    （在本课程的后半部分，你将会学到 `Relaxed` 是足够的。
    But don't use `Relaxed` in this homework, because that would break `cargo_tsan`.)
    但是不要在这个作业中使用`Relaxed`，因为那会破坏`cargo_tsan`。
  * Since a node can be removed while another thread is reading,
    由于在另一个线程读取时节点可能被移除，
    reclamation of the node should be deferred.
    节点的回收应当被推迟。
    You can handle this semi-automatically with `crossbeam_epoch`.
    你可以使用 `crossbeam_epoch` 半自动处理这个。

**Instruction**: Fill in the `todo!()`s in `list_set/optimistic_fine_grained.rs` (about 80 lines of code).
**说明**：在 `list_set/optimistic_fine_grained.rs` 中填写 `todo!()`（约 80 行代码）。

**Testing**: Tests are defined in `tests/list_set/optimistic_fine_grained.rs`.
**测试**：测试在 `tests/list_set/optimistic_fine_grained.rs` 中定义。

**Self grading**:
**自我评分**:
Run
跑
```
./scripts/grade-optimistic_list_set.sh
```

Unlike the main homework, the grader additionally runs the following tests
与主要作业不同，评分脚本还会运行以下测试
(10 points if all of them passes, otherwise 0).
（如果全部通过得10分，否则得0分）。
* `read_no_block`
* `iter_invalidate_end`
* `iter_invalidate_deleted`
