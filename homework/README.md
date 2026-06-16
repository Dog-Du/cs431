# Tips
> 提示

- Read the paper and the skeleton code carefully.  I'll ask questions about those in the exams.
  仔细阅读论文和骨架代码。我会在考试中问关于它们的问题。

- Read [the Rust book](https://doc.rust-lang.org/book/), especially the ["getting started"
  阅读 [the Rust book](https://doc.rust-lang.org/book/)，特别是 [“入门”]
  section](https://doc.rust-lang.org/book/ch01-00-getting-started.html) for learning how to build
  部分](https://doc.rust-lang.org/book/ch01-00-getting-started.html) 用于学习如何构建
  and test the development.
  并测试开发。

- Use [Visual Studio Code](https://code.visualstudio.com/) or
  使用 [Visual Studio Code](https://code.visualstudio.com/) 或
  [CLion](https://www.jetbrains.com/clion/) for interactive debugging.  The former is free of charge
  [CLion](https://www.jetbrains.com/clion/) 用于交互式调试。前者是免费的
  for everyone, and The latter is [free of charge for students](https://www.jetbrains.com/student/).
  对每个人来说，后者是 [free of charge for students](https://www.jetbrains.com/student/)。
    + [Manual for debugging rust code in
      [Rust 代码调试手册
      VSCode](https://www.forrestthewoods.com/blog/how-to-debug-rust-with-visual-studio-code/)
      (using [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)
      (使用 [CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)
      plugin)
      插件）
    + [Manual for debugging rust code in
      [Rust 代码调试手册
      CLion](https://www.jetbrains.com/help/clion/rust-support.html)

- Use rustfmt and clippy:
  使用 rustfmt 和 clippy：

  ```sh
  cargo fmt
  cargo clippy
  ```

- Running individual tests
  运行单个测试

  ```sh
  # Run all tests in a module
  cargo test --test <module name>
  # For example, run all tests in tests/hazard_pointer.rs
  cargo test --test hazard_pointer

  # Run all tests in a module that matches (substring) the name
  cargo test --test <module name> <test name>
  # For example, run the stack_queue test in the hazard_pointer module
  cargo test --test hazard_pointer stack_queue

  # Run the test that exactly matches the name
  cargo test --test <module name> -- --exact <test name>
  ```

- Running grading scripts in Mac: [#338](https://github.com/kaist-cp/cs431/issues/338).
  在 Mac 上运行评分脚本：[#338](https://github.com/kaist-cp/cs431/issues/338)。

- Q: Sanitizer output is not readable.
  问：消毒剂输出不可读。
  A: Make sure that `llvm-symbolizer` is under `$PATH`.
  A：确保 `llvm-symbolizer` 在 `$PATH` 下面。
  ```
  sudo ln -s /usr/bin/llvm-symbolizer-14 /usr/bin/llvm-symbolizer
  ```
  (Adjust "-14" part based on the llvm version installed on your system.)
  （根据你系统上安装的 llvm 版本调整“-14”部分。）

## Using LLVM Sanitizers
> 使用 LLVM Sanitizers

We use LLVM sanitizers for grading.
我们使用 LLVM 诊断工具进行评分。
Sanitizers are dynamic analysis tools that detect buggy behaviors during runtime. For example,
消毒器是动态分析工具，能够在运行时检测有缺陷的行为。例如，
[AddressSanitizer](https://clang.llvm.org/docs/AddressSanitizer.html) detects memory bugs like use-after-free and
[AddressSanitizer](https://clang.llvm.org/docs/AddressSanitizer.html) 检测内存错误，例如使用后释放（use-after-free）和
[ThreadSanitizer](https://clang.llvm.org/docs/ThreadSanitizer.html) detects data races.
[ThreadSanitizer](https://clang.llvm.org/docs/ThreadSanitizer.html) 检测数据竞争。

You can run the tests with sanitizers using the following commands:
您可以使用以下命令运行带有清理器的测试：
```sh
source scripts/grade-utils.sh
# This may take some time because of `rustup toolchain update stable nightly` in the script.
# If you have run that already, please feel free to comment that line out.

cargo_asan SUBCOMMAND
# cargo_asan runs the following command
# RUSTFLAGS="-Z sanitizer=address" cargo +nightly SUBCOMMAND --target x86_64-unknown-linux-gnu

# For example, run all tests in the hazard_pointer module under the address sanitizer
cargo_asan test --test hazard_pointer

cargo_tsan SUBCOMMAND
# cargo_tsan runs the following command
# TSAN_OPTIONS="suppressions=suppress_tsan.txt" RUST_TEST_THREADS=1 RUSTFLAGS="-Z sanitizer=thread" cargo +nightly SUBCOMMAND --target x86_64-unknown-linux-gnu
# (`suppressions=suppress_tsan.txt` is for suppressing some false positive from ThreadSanitizer.)

# For example, run all tests in the growable_array module under the thread sanitizer
cargo_tsan test --test growable_array
```

While (safe) Rust's type system guarantees memory safety and the absence of data race,
虽然（安全的）Rust 类型系统保证了内存安全和没有数据竞争，
this guarantee relies on the correctness of the libraries implemented with unsafe features.
这种保证依赖于使用不安全功能实现的库的正确性。
Therefore, tools like sanitizers are still essential when we use unsafe Rust.
因此，当我们使用不安全的 Rust 时，像消毒器这样的工具仍然是必不可少的。
