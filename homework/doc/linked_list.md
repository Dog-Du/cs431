# Doubly linked list
> 双向链表
**Implement doubly linked list in unsafe Rust.**
**在 unsafe Rust 中实现双向链表。**

This homework serves as a brief tutorial for unsafe Rust with a focus on the basic raw pointer operations.
这份作业作为一个关于不安全 Rust 的简短教程，重点是基本的裸指针操作。

The [skeleton code](https://github.com/kaist-cp/cs431/blob/main/homework/src/linked_list.rs) is slightly modified version of [the linked list from Rust standard library](https://doc.rust-lang.org/std/collections/struct.LinkedList.html).
[骨架代码](https://github.com/kaist-cp/cs431/blob/main/homework/src/linked_list.rs) 是 [Rust 标准库中的链表](https://doc.rust-lang.org/std/collections/struct.LinkedList.html) 的稍微修改版本。
We already provided implementation for several methods e.g. `push_front_node`.
我们已经为几个方法提供了实现，例如 `push_front_node`。
Your job is to implement their symmetric counterparts
你的工作是实现它们的对称对应物
e.g. `push_back_node` and some methods of `IterMut` struct (see `todo!()`s).
例如 `push_back_node` 以及 `IterMut` 结构的一些方法（见 `todo!()`s）。

You can look up its implementation from the standard library,
你可以从标准库中查找它的实现，
but we encourage you do it yourself
但是我们鼓励你自己做
so that you can build enough skill set for upcoming homeworks.
这样你就可以为即将到来的作业建立足够的技能组合。
We also recommend you to play around with AddressSanitizer and debugger.
我们也建议你尝试使用 AddressSanitizer 和调试器。

## Grading
> 评分
* The full score for this homework is 40 points (HW1 was 100) and the total lines of code to be written is about 80.
  这个作业的满分是40分（HW1是100分），需要编写的总代码行数约为80行。
* You can evaluate your solution by running `./scripts/grade-linked_list.sh` in the `homework` directory.
  您可以通过在 `homework` 目录中运行 `./scripts/grade-linked_list.sh` 来评估您的解决方案。

## Submission
> 提交
Submit `linked_list.rs` to gg.
将 `linked_list.rs` 提交给 gg。

## Guide
> 指南

### Learn the basics of unsafe Rust
> 学习 unsafe Rust 的基础
1. Read [Rust Book §19.1](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html)
   阅读 [Rust Book §19.1](https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html)
1. Skim through [Nomicon §1](https://doc.rust-lang.org/nomicon/meet-safe-and-unsafe.html)
   浏览 [Nomicon §1](https://doc.rust-lang.org/nomicon/meet-safe-and-unsafe.html)
1. Read [raw pointer type documentation](https://doc.rust-lang.org/std/primitive.pointer.html) and some of its methods (`is_null`, `as_ref`, `read`, `write`, `replace`, `swap`)
   阅读 [raw pointer type documentation](https://doc.rust-lang.org/std/primitive.pointer.html) 及其一些方法（`is_null`、`as_ref`、`read`、`write`、`replace`、`swap`）
1. Read [`std::mem`](https://doc.rust-lang.org/std/mem/index.html) and [`std::ptr`](https://doc.rust-lang.org/std/ptr/index.html) documentations.
   阅读 [`std::mem`](https://doc.rust-lang.org/std/mem/index.html) 和 [`std::ptr`](https://doc.rust-lang.org/std/ptr/index.html) 文档。
1. Read [`std::iter`](https://doc.rust-lang.org/std/iter/index.html) documentation.
   阅读 [`std::iter`](https://doc.rust-lang.org/std/iter/index.html) 文档。

### Tips for debugging
> 调试提示
When `cargo test` fails with error messages like this,
当 `cargo test` 出现类似这样的错误信息时，
```
thread panicked while panicking. aborting.
error: test failed, to rerun pass '--test linked_list'

Caused by:
  process didn't exit successfully: ... (signal: 4, SIGILL: illegal instruction)
```
try running the test like this
试着这样运行测试
<pre>
cargo test --test linked_list <strong>-- --nocapture --test-threads 1</strong>
</pre>
This will give you more informative error messages.
这将为您提供更详细的错误信息。

### Other useful resources
> 其他有用资源
* [`*` operator](https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html#the-dereference-operator)
  [`*` 操作符](https://doc.rust-lang.org/stable/reference/expressions/operator-expr.html#the-dereference-operator)
* [`.` operator](https://doc.rust-lang.org/stable/reference/expressions/call-expr.html)
  [`.` 操作符](https://doc.rust-lang.org/stable/reference/expressions/call-expr.html)
* [type coercion (weakening)](https://doc.rust-lang.org/nomicon/coercions.html)
  [类型强制（弱化）](https://doc.rust-lang.org/nomicon/coercions.html)
* [type casting](https://doc.rust-lang.org/nomicon/casts.html)
  [类型转换](https://doc.rust-lang.org/nomicon/casts.html)
* [`Box<T>` is special](https://doc.rust-lang.org/stable/reference/special-types-and-traits.html#boxt)
  [`Box<T>` 很特别](https://doc.rust-lang.org/stable/reference/special-types-and-traits.html#boxt)
* [`*const T` vs. `*mut T`](https://internals.rust-lang.org/t/what-is-the-real-difference-between-const-t-and-mut-t-raw-pointers/6127)
  [`*const T` 对 `*mut T`](https://internals.rust-lang.org/t/what-is-the-real-difference-between-const-t-and-mut-t-raw-pointers/6127)
