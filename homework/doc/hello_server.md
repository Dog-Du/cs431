# Parallel web server with cache
> 带缓存的并行 Web 服务器

## Expected outcome
> 预期结果

- Execute `cargo run --features="build-bin" hello_server`. A web server should run. If it doesn't, try changing the port used in [`hello_server.rs:6`](../src/bin/hello_server.rs).
  执行 `cargo run --features="build-bin" hello_server`。Web 服务器应运行。如果没有，请尝试更改 [`hello_server.rs:6`](../src/bin/hello_server.rs) 中使用的端口。
- Run `curl http://localhost:7878/alice`. It should wait for a few seconds, and return a web page.
  运行 `curl http://localhost:7878/alice`。它应该等待几秒钟，然后返回一个网页。
- Run `curl http://localhost:7878/alice` again. It should instantly return a web page.
  再次运行 `curl http://localhost:7878/alice`。它应该会立即返回一个网页。
- Run `curl http://localhost:7878/bob`. It should wait for a few seconds, and return a web page.
  运行 `curl http://localhost:7878/bob`。它应该等待几秒钟，然后返回一个网页。
- Press `Ctrl-C`. The web server should gracefully shut down after printing statistics.
  按下 `Ctrl-C`。网络服务器在打印统计信息后应优雅地关闭。

## Organization
> 组织结构

- `../src/bin/hello_server.rs`: the web server.
  `../src/bin/hello_server.rs`：网络服务器。
- `../src/hello_server/*.rs`: the server components. You should fill out `todo!()` in those files.
  `../src/hello_server/*.rs`：服务器组件。你应该在那些文件中填写 `todo!()`。

## Grading
> 评分
The grader runs `./scripts/grade-hello_server.sh` in the `homework` directory.
评分脚本在 `homework` 目录中运行 `./scripts/grade-hello_server.sh`。
This script runs the tests with various options.
这个脚本使用各种选项运行测试。

There will be no partial scores for `tcp` and `thread_pool` modules.
`tcp` 和 `thread_pool` 模块将不会有部分分数。
That is, you will get the score for a module only if your implementation passes **all** tests for that module.
也就是说，只有当你的实现通过该模块的**所有**测试时，你才会获得该模块的分数。

On the other hand, we will give partial scores for `cache` module.
另一方面，我们将对 `cache` 模块给予部分分数。
In particular, even if your implementation of `cache` blocks concurrent accesses to different keys, you can still get some points for basic functionalities.
特别是，即使你对 `cache` 的实现阻止了对不同键的并发访问，你仍然可以因为基本功能获得一些分数。

## Submission
> 提交
```bash
cd cs431/homework
./scripts/submit.sh
ls ./target/hw-hello_server.zip
```
Submit `hw-hello_server.zip` to gg.
将 `hw-hello_server.zip` 提交给 gg。

## Guide
> 指南

### Reading Rust book
> 阅读 Rust 书
This homework requires a good understanding of the materials covered in [the Rust book §20](https://doc.rust-lang.org/book/ch20-00-final-project-a-web-server.html).
这个作业需要对 [the Rust book §20](https://doc.rust-lang.org/book/ch20-00-final-project-a-web-server.html) 中涵盖的材料有良好的理解。
This is the minimal path for understanding §20: §1, 2, 3, 4, 5, 6, 8, 9, 10, 13.1, 13.2, **15**, **16**, **20**.
这是理解 §20 的最小路径：§1、2、3、4、5、6、8、9、10、13.1、13.2、**15**、**16**、**20**。

Specifically, make sure that you understand the following topics.
具体来说，确保你理解以下主题。
* [`Drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html) trait and [`drop`](https://doc.rust-lang.org/std/mem/fn.drop.html) function
  [`Drop`](https://doc.rust-lang.org/std/ops/trait.Drop.html) 特性和 [`drop`](https://doc.rust-lang.org/std/mem/fn.drop.html) 功能
* Type signature of [`std::thread::spawn`](https://doc.rust-lang.org/std/thread/fn.spawn.html) and the meaning of [`std::thread::JoinHandle`](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html).
  [`std::thread::spawn`](https://doc.rust-lang.org/std/thread/fn.spawn.html) 的类型签名以及 [`std::thread::JoinHandle`](https://doc.rust-lang.org/std/thread/struct.JoinHandle.html) 的含义。
* The meaning and usage of [`Arc<`](https://doc.rust-lang.org/std/sync/struct.Arc.html)[`Mutex<T>>`](https://doc.rust-lang.org/std/sync/struct.Mutex.html).
  [`Arc<`](https://doc.rust-lang.org/std/sync/struct.Arc.html)[`Mutex<T>>`](https://doc.rust-lang.org/std/sync/struct.Mutex.html) 的意义和用法。
* [Channels](https://doc.rust-lang.org/std/sync/mpsc/index.html).
  [频道](https://doc.rust-lang.org/std/sync/mpsc/index.html)
<!-- * The fact that there is no non-trivial way to break out of `TcpListener::incoming` loop. -->
<!-- * 事实上没有非平凡的方法可以跳出 `TcpListener::incoming` 循环。 -->

See also: Rust book with quiz. <https://rust-book.cs.brown.edu/>
另请参阅：带测验的 Rust 书。 <https://rust-book.cs.brown.edu/>

### Major differences between HW1 thread pool and Rust book §20 thread pool
> HW1 线程池与 Rust 书 §20 线程池的主要区别
1. We use [`crossbeam_channel`](https://docs.rs/crossbeam-channel/) instead of [<code>std::sync::<strong>mpsc</strong></code>](https://doc.rust-lang.org/std/sync/mpsc/index.html). Since crossbeam's channels are **mpmc**, you don't need to wrap the `Receiver` inside a `Mutex`.
   我们使用 [`crossbeam_channel`](https://docs.rs/crossbeam-channel/) 而不是 [<code>std::sync::<strong>mpsc</strong></code>](https://doc.rust-lang.org/std/sync/mpsc/index.html)。由于 crossbeam 的通道是 **mpmc**，你不需要将 `Receiver` 包装在 `Mutex` 中。
1. We do not use explicit exit messages for the thread pool. Instead, we disconnect the channel by `drop`ping the receiver/sender.
   我们不对线程池使用明确的退出消息。相反，我们通过 `drop`ping 接收器/发送器来断开通道。
    * Our message type is simply the `Job` itself:
      我们的消息类型就是 `Job` 本身：
      ```rust
      struct Job(Box<dyn FnOnce() + Send + 'static>);
      ```
    * Each worker thread automatically breaks out of the loop if the channel is disconnected.
      如果通道断开，每个工作线程会自动跳出循环。
1. We `join()` each thread in the destructor of `Worker`, not in the destructor of `ThreadPool`. Since `ThreadPool` has field `workers: Vec<Worker>`, the worker destructor will be called when the pool is dropped. Note that the channel should be disconnected before `join()`ning the worker threads. (Otherwise, `join` will block.) This means that the `Sender` should be dropped before `Vec<Worker>`. You can specify the drop order in many ways. In this homework, we use `ThreadPool::job_sender` of type `Option<Sender<Job>>`, whose content can be `take()`n and `drop()`ped explicitly in `<ThreadPool as Drop>::drop`.
   我们在 `Worker` 的析构函数中而不是在 `ThreadPool` 的析构函数中 `join()` 每个线程。由于 `ThreadPool` 有字段 `workers: Vec<Worker>`，当池被释放时，工作线程的析构函数将被调用。请注意，在 `join()` 工作线程之前，通道应该先断开。（否则，`join` 将阻塞。）这意味着 `Sender` 应该在 `Vec<Worker>` 之前被释放。你可以通过多种方式指定释放顺序。在本作业中，我们使用类型为 `Option<Sender<Job>>` 的 `ThreadPool::job_sender`，其内容可以在 `<ThreadPool as Drop>::drop` 中被 `take()`n 和 `drop()`ped 显式释放。

### Tips
> 提示
* Cache: Start with `Mutex<HashMap<K, V>>`. To fully implement the specification, you will need a more complicated type. The simplest solution makes use of all the things imported in `cache.rs`.
  缓存：从 `Mutex<HashMap<K, V>>` 开始。要完全实现规范，您将需要一种更复杂的类型。最简单的解决方案是利用在 `cache.rs` 中导入的所有内容。
* Interrupt handler: just follow the comments.
  中断处理程序：只需遵循注释。
* Thread pool: Ignore `ThreadPoolInner` first (it's used for `ThreadPool::join`), and implement the changes discussed above.
  线程池：首先忽略 `ThreadPoolInner`（它用于 `ThreadPool::join`），并实现上面讨论的更改。
* If you have questions, try looking up the [issue tracker](https://github.com/kaist-cp/cs431/issues).
  如果你有问题，试着查找 [issue 跟踪器](https://github.com/kaist-cp/cs431/issues)。
  There are many Q&A's from the previous iterations of this course, and they are labeled by the topic.
  这个课程之前的版本有很多问答，并且它们按主题标注。
  For example, ["homework - cache" label](https://github.com/kaist-cp/cs431/issues?q=label%3A%22homework+-+cache%22+) lists the questions about `cache.rs`.
  例如，["homework - cache" label](https://github.com/kaist-cp/cs431/issues?q=label%3A%22homework+-+cache%22+) 列出了关于 `cache.rs` 的问题。
  Here are some Q&A's you may find useful for this homework:
  这里有一些你可能会觉得对这个作业有用的问答：
    * https://github.com/kaist-cp/cs431/issues/339
    * https://github.com/kaist-cp/cs431/issues/85#issuecomment-696888546
    * https://github.com/kaist-cp/cs431/issues/81

### Testing
> 测试
We'll only test the libraries.
我们只会测试这些库。
```bash
cargo test --test cache
cargo test --test tcp
cargo test --test thread_pool
```
We will use those tests for grading, too. We may add some more tests for grading, but if your solution passes all the given tests, you will likely get the full score.
我们也会使用那些测试进行评分。我们可能会增加一些用于评分的测试，但如果你的解决方案通过了所有给定的测试，你很可能会得到满分。

Also, try running tests with the [LLVM sanitizers](https://github.com/kaist-cp/cs431/tree/main/homework#using-llvm-sanitizers) enabled.
另外，尝试在启用 [LLVM sanitizers](https://github.com/kaist-cp/cs431/tree/main/homework#using-llvm-sanitizers) 的情况下运行测试。
They are not that useful for HW1, but they will be very helpful for upcoming homework assignments.
它们对作业1不是很有用，但对即将到来的作业会非常有帮助。
