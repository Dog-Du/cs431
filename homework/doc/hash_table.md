# Lock-free hashtable
> 无锁哈希表
**Implement a lock-free hash table based on recursive split-ordered list.**
**基于递归分割有序列表实现无锁哈希表。**

This homework is in 2 parts:
这个作业分为两部分：
1. (140 points) Functionality:
   (140 分) 功能:
   Implement the hash table with `Ordering::SeqCst` for every atomic access.
   使用 `Ordering::SeqCst` 为每次原子访问实现哈希表。
   With this, we can pretend that we are on a sequentially consistent memory model.
   有了这个，我们可以假装我们在一个顺序一致内存模型上。
2. (40 points) Performance:
   (40 分) 性能：
   After you learn about relaxed memory semantics,
   在你了解了松弛内存语义之后，
   optimize the implementation by relaxing the ordering on the atomic accesses.
   通过放宽原子访问的内存序来优化实现。
   We recommend working on this part after finishing the [Arc homework](./arc.md).
   我们建议在完成 [Arc 作业](./arc.md) 之后再处理这一部分。

## ***2024 spring semester notice: Part 2 is cancelled***
> ***2024年春季学期通知：第二部分取消***
We won't cover the weak memory semantics in this semester.
本学期我们不会讲弱内存语义。

## Part 1: Split-ordered list in sequentially consistent memory model
> 第1部分：顺序一致内存模型中的分裂有序列表
1. Fully understand the following reading materials.
   完全理解以下阅读材料。
    + [The original paper on the split-ordered list](https://dl.acm.org/doi/abs/10.1145/1147954.1147958).
      [关于分裂有序列表的原始论文](https://dl.acm.org/doi/abs/10.1145/1147954.1147958)
      You can skip the correctness proof and performance evaluation section.
      你可以跳过正确性证明和性能评估部分。
      Alternatively, read the chapter 13.3 of [The Art of Multiprocessor Programming](https://dl.acm.org/doi/book/10.5555/2385452).
      或者，阅读 [The Art of Multiprocessor Programming](https://dl.acm.org/doi/book/10.5555/2385452) 的第 13.3 章。
      It presents the same stuff, but is more readable.
      它提供了相同的内容，但更易读。
    + The [lock-free linked list](https://github.com/kaist-cp/cs431/blob/main/src/lockfree/list.rs) interface and implementation.
      [无锁链表](https://github.com/kaist-cp/cs431/blob/main/src/lockfree/list.rs) 接口及其实现。
1. Implement `GrowableArray` in [`hash_table/growable_array.rs`](../src/hash_table/growable_array.rs). (about 100 LOC)
   在 [`hash_table/growable_array.rs`](../src/hash_table/growable_array.rs) 中实现 `GrowableArray`。（约 100 行代码）
    * You'll need to properly use [Rust `union`s](https://doc.rust-lang.org/reference/items/unions.html).
      你需要正确使用 [Rust `union`s](https://doc.rust-lang.org/reference/items/unions.html)。
    * To represent the height of the segment tree, [tag](https://en.wikipedia.org/wiki/Tagged_pointer) the `root` pointer with the height.
      为了表示线段树的高度，用[tag](https://en.wikipedia.org/wiki/Tagged_pointer)指针表示`root`高度。
      Use [`tag`](https://docs.rs/crossbeam/*/crossbeam/epoch/struct.Shared.html#method.tag) and [`with_tag`](https://docs.rs/crossbeam/*/crossbeam/epoch/struct.Shared.html#method.with_tag).
      使用 [`tag`](https://docs.rs/crossbeam/*/crossbeam/epoch/struct.Shared.html#method.tag) 和 [`with_tag`](https://docs.rs/crossbeam/*/crossbeam/epoch/struct.Shared.html#method.with_tag)。
      See [`lockfree/list.rs`](https://github.com/kaist-cp/cs431/blob/main/src/lockfree/list.rs) for example usage.
      参见 [`lockfree/list.rs`](https://github.com/kaist-cp/cs431/blob/main/src/lockfree/list.rs) 获取示例用法。
      See also: [#226](https://github.com/kaist-cp/cs431/issues/226)
      另请参见：[#226](https://github.com/kaist-cp/cs431/issues/226)
1. Implement `SplitOrderedList` in [`hash_table/split_ordered_list.rs`](../src/hash_table/split_ordered_list.rs). (about 80 LOC)
   在 [`hash_table/split_ordered_list.rs`](../src/hash_table/split_ordered_list.rs) 中实现 `SplitOrderedList`。（约 80 行代码）
    * You can use bitwise operations on `usize` e.g. `<<`, `&`, `|`, `^`, ...
      你可以对 `usize` 使用按位运算，例如 `<<`、`&`、`|`、`^`，...
      See also: [`leading_zeros`](https://doc.rust-lang.org/std/primitive.usize.html#method.leading_zeros), [`reverse_bits`](https://doc.rust-lang.org/std/primitive.usize.html#method.reverse_bits), [`size_of`](https://doc.rust-lang.org/std/mem/fn.size_of.html)
      另见：[`leading_zeros`](https://doc.rust-lang.org/std/primitive.usize.html#method.leading_zeros), [`reverse_bits`](https://doc.rust-lang.org/std/primitive.usize.html#method.reverse_bits), [`size_of`](https://doc.rust-lang.org/std/mem/fn.size_of.html)
    * We provided type signatures for 2 helper methods for `SplitOrderedList`.
      我们为 `SplitOrderedList` 提供了两个辅助方法的类型签名。
      You can modify/remove them or add more private methods if you want to.
      如果你愿意，你可以修改/删除它们或添加更多私有方法。
      Just make sure you don't change the public interface. You can import other stuff from the `core` or `crossbeam_epoch` crates (but not necessary).
      只要确保你不改变公共接口。你可以从 `core` 或 `crossbeam_epoch` 包中导入其他内容（但不是必须的）。

## Part 2: Relaxing the orderings
> 第 2 部分：放宽内存序
Use release-acquire synchronization for atomic accesses, just like many other data structures covered in the lecture.
对于原子访问，使用release-acquire 同步，就像讲座中涉及的许多其他数据结构一样。


## Testing
> 测试
Tests are defined in `tests/{growable_array,hash_table}.rs`.
测试在 `tests/{growable_array,hash_table}.rs` 中定义。
They use the common map test functions defined in `src/test/adt/map.rs`.
他们使用在 `src/test/adt/map.rs` 中定义的常用映射测试函数。

## Grading (180 points)
> 评分（180分）
Run `./scripts/grade-hash_table.sh`.
运行 `./scripts/grade-hash_table.sh`。

### Part 1: Functionality (140 points)
> 第1部分：功能性（140分）
For each module `growable_array` and `split_ordered_list`,
对于每个模块 `growable_array` 和 `split_ordered_list`，
the grader runs the tests with `cargo`, `cargo_asan`, and `cargo_tsan` in the following order.
评分脚本按以下顺序使用 `cargo`、`cargo_asan` 和 `cargo_tsan` 运行测试。
1. `stress_sequential` (5 points)
   `stress_sequential`（5 分）
1. `lookup_concurrent` (5 points)
   `lookup_concurrent`（5 分）
1. `insert_concurrent` (10 points)
   `insert_concurrent`（10 分）
1. `stress_concurrent` (20 points)
   `stress_concurrent`（20 分）
1. `log_concurrent` (30 points)
   `log_concurrent`（30 分）

Note:
注意：
* If a test fails in a module, then the later tests in the same module will not be run.
  如果一个模块中的测试失败，那么同一模块中的后续测试将不会运行。
* The test timeout is at least 5x of the time our implementation took on the homework server.
  测试超时时间至少是我们在作业服务器上实现所花时间的5倍。
  It is not a tight timeout, but it will detect clearly incorrect implementations.
  这不是严格的超时，但它能够清楚地检测出错误的实现。

### Part 2: Relaxed ordering (40 points)
> 第2部分：Relaxed 内存序（40分）
For each module `growable_array` and `split_ordered_list`,
对于每个模块 `growable_array` 和 `split_ordered_list`，
the grader checks the usage of `SeqCst` ordering and gives 20 points if it is not used.
评分脚本会检查 `SeqCst` 内存序的使用情况，如果没有使用则给 20 分。

Since `split_ordered_list` uses `growable_array`, using `SeqCst` in `growable_array` means it
由于 `split_ordered_list` 使用 `growable_array`，在 `growable_array` 中使用 `SeqCst` 意味着它
is used in `split_ordered_list` as well.
也用于 `split_ordered_list`。

## Submission
> 提交
```bash
cd cs431/homework
./scripts/submit.sh
ls ./target/hw-hash_table.zip
```
Submit `hw-hash_table.zip` to gg.
将 `hw-hash_table.zip` 提交给 gg。
