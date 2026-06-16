# Behaviour-Oriented Concurrency (BoC)
> 面向行为的并发 (BoC)
**Implement a runtime for Behaviour-Oriented Concurrency**
**实现一个用于行为导向并发的运行时**

> *The Behaviour-Oriented Concurrency paradigm: a concurrency paradigm
> *面向行为的并发范式：一种并发范式
> that achieves flexible coordination over multiple resources, and ordered execution, and scalability.* (from §1 of the BoC [paper](https://doi.org/10.1145/3622852))
> 实现对多个资源的灵活协调、有序执行和可扩展性。*（摘自 BoC [论文](https://doi.org/10.1145/3622852) 第1节）

First, read [the original BoC paper](https://doi.org/10.1145/3622852) and understand its algorithm.
首先，阅读 [原始 BoC 论文](https://doi.org/10.1145/3622852) 并理解它的算法。
In particular, you should understand the key concepts (e.g., cown, behaviour, when, and thunk), and fully understand `Fig.3` and `§4.3` which contain the details of the implementation.
特别是，你应该理解关键概念（例如，cown、行为、when 和 thunk），并且完全理解 `Fig.3` 和 `§4.3`，它们包含了实现的详细信息。

Fill in the `todo!()`s in `src/boc.rs`.
在 `src/boc.rs` 中填写 `todo!()`。
The total lines of code to be written is about 70.
要编写的代码总行数大约是70行。
Your implementation should satisfy the following criterias:
你的实现应满足以下标准：
* when clauses should be scheduled in the correct order of the *dependency graph* (§4.1).
  when 子句应该按照*依赖图*（§4.1）的正确顺序安排时。
* Your implementation of the BoC runtime should ensure *deadlock freedom*.
  BoC 运行时的实现应保证 *无死锁性*。
  We will test the deadlock freedom by several stress tests with timeouts.
  我们会用多个带超时的压力测试来检查无死锁性。
* Whenever you want to spawn a new thread, **don't use** [`std::thread::spawn`](https://doc.rust-lang.org/std/thread/fn.spawn.html).
  每当你想创建一个新线程时，**不要使用** [`std::thread::spawn`](https://doc.rust-lang.org/std/thread/fn.spawn.html)。
  Instead, use [`rayon::spawn`](https://docs.rs/rayon/latest/rayon/fn.spawn.html).
  相反，使用 [`rayon::spawn`](https://docs.rs/rayon/latest/rayon/fn.spawn.html)。

We provide several ways of using the when clause in Rust, illustrated below.
我们提供了几种在 Rust 中使用 when 子句的方法，示例如下。

1.  Using the `when!` macro. Below is a representative example describing its use:
   使用 `when!` 宏。下面是描述其使用的一个示例：

    ```rust
    when!(c1, c2; g1, g2; {
        ... // thunk
    });
    ```
    This results in a when clause that schedules a new behavior for two `CownPtr`s `c1` and `c2`.
    这会导致一个当子句，为两个 `CownPtr`s `c1` 和 `c2` 安排新的行为。
    `g1` and `g2` are mutable references to the shared resources protected by given `CownPtr`s
    `g1` 和 `g2` 是对由给定 `CownPtr` 保护的共享资源的可变引用
    and can be used in the thunk.
    并且可以在 thunk 中使用。
2.  Using the `run_when` function directly. Use this if you want to create a new behavior with an arbitrary number of `CownPtr`s.
   直接使用 `run_when` 函数。如果你想用任意数量的 `CownPtr` 创建新的行为，请使用此方法。
    For example,
    例如，

    ```rust
    run_when(vec![c1.clone(), c2.clone(), c3.clone()], move |mut acc| {
        ... // thunk
    });
    ```
    The first argument is a `Vec` of cowns with the same type.
    第一个参数是具有相同类型的 cowns 的 `Vec`。
    `acc` is a vector of mutable references to the shared resources protected by the cowns,
    `acc` 是一个可变引用向量，指向由 cowns 保护的共享资源，
    and it is guaranteed that `acc` has the same length as the specified given `Vec` of `CownPtr`s.
    并且可以保证 `acc` 的长度与指定的给定 `Vec` 的 `CownPtr` 相同。

More examples can be found in `src/boc.rs` and `test/boc.rs`.
更多例子可以在 `src/boc.rs` 和 `test/boc.rs` 中找到。

## Grading (100 points)
> 评分（100分）
Run `./scripts/grade-boc.sh`.
运行 `./scripts/grade-boc.sh`。
Basic tests account for 60 points and stress tests account for 40 points.
基础测试占60分，压力测试占40分。

Note: You don't need to worry about the message (shown below) that might be printed during the tests with `cargo_tsan`.
注意：你不需要担心在使用 `cargo_tsan` 进行测试时可能出现的消息（如下所示）。
It will not affect the grading.
它不会影响评分。
```
/usr/bin/addr2line: DWARF error: invalid or unhandled FORM value: 0x23
```

## Submission
> 提交
Submit `boc.rs` to gg.
将 `boc.rs` 提交给 gg。
