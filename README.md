# KAIST CS431: Concurrent Programming
> KAIST CS431：并发编程

## Logistics
> 课程事务

- Instructor: [Jeehoon Kang](https://www.fearless.systems/jeehoon.kang)
  讲师：[Jeehoon Kang](https://www.fearless.systems/jeehoon.kang)
- Time: Mon & Wed 13:00-14:15 (2024 Spring)
  时间：周一&周三 13:00-14:15（2024春季）
- Place
  地点
  + Rm. 1101, Bldg. E3-1. **YOUR PHYSICAL ATTENDANCE IS REQUIRED** unless announced otherwise.
    1101 室，E3-1 栋。**除非另行通知，否则需要您亲自到场**。
  + [Zoom room](https://kaist.zoom.us/my/jeehoon.kang) (if remote participation is absolutely necessary).
    [Zoom 房间](https://kaist.zoom.us/my/jeehoon.kang)（如果绝对有必要远程参与）。
    The passcode is announced at KLMS.
    通行码将在KLMS上公布。
  + [Youtube channel](https://www.youtube.com/playlist?list=PL5aMzERQ_OZ9j40DJNlsem2qAGoFbfwb4).
    [YouTube 频道](https://www.youtube.com/playlist?list=PL5aMzERQ_OZ9j40DJNlsem2qAGoFbfwb4)
    Turn on English subtitles on YouTube, if necessary.
    如有必要，请在YouTube上打开英文字幕。
- Websites: <https://github.com/kaist-cp/cs431>, <https://gg.kaist.ac.kr/course/19>
  网站: <https://github.com/kaist-cp/cs431>, <https://gg.kaist.ac.kr/course/19>
- Announcements: in the [issue tracker](https://github.com/kaist-cp/cs431/issues?q=is%3Aissue+is%3Aopen+label%3Aannouncement)
  公告: 在 [issue 跟踪器](https://github.com/kaist-cp/cs431/issues?q=is%3Aissue+is%3Aopen+label%3Aannouncement)
  + We assume that you will read each announcement within 24 hours.
    我们假设您将在24小时内阅读每条公告。
  + We strongly recommend you watch the repository.
    我们强烈建议您关注该仓库。
- TA: [Sunho Park](https://www.fearless.systems/sunho.park/) (Head TA), [Janggun Lee](https://www.fearless.systems/janggun.lee/).
  助教：[Sunho Park](https://www.fearless.systems/sunho.park/)（首席助教），[Janggun Lee](https://www.fearless.systems/janggun.lee/)。
  + Office Hours: Fri 9:15-10:15, Rm. 4432, Bldg. E3-1.
    答疑时间：星期五 9:15-10:15，E3-1 楼 4432 室。
    If you want to come, do so by 9:30.
    如果你想来，请在9:30之前来。
    See [below](https://github.com/kaist-cp/cs431#rules) for the office hour policy.
    有关办公时间政策，请参阅 [下文](https://github.com/kaist-cp/cs431#rules)。
    <!-- Fri 9:00-12:00, [Zoom room](https://zoom.us/j/4842624821)(The passcode is same as the class). It is not required, but if you want to come, do so by 9:30. See [below](#communication) for office hour policy. -->
    <!-- 周五 9:00-12:00, [Zoom 房间](https://zoom.us/j/4842624821)（密码与课程相同）。不是必须的，但如果你想来，请在9:30之前到。办公室时间政策见 [下文](#communication)。 -->
- **IMPORTANT**: you should not expose your work to others. In particular, you should not fork the [upstream](https://github.com/kaist-cp/cs431) and push there.
  **重要**：你不应该将你的工作暴露给他人。特别是，你不应该分叉 [上游仓库](https://github.com/kaist-cp/cs431) 并在那里推送。


## Course description
> 课程描述

### Context
> 背景

I anticipate that in the next 700 years, computers will be **massively parallel**.
我预计在未来700年，计算机将是**大规模并行**的。
Humankind seeks to enhance computer performance in the era of big data.
人类在大数据时代寻求提升计算机性能。
This goal has become increasingly challenging following the breakdown of [Dennard scaling](https://en.wikipedia.org/wiki/Dennard_scaling) around 2005, indicating that the performance of sequential computers is unlikely to improve further.
自从[Dennard scaling](https://en.wikipedia.org/wiki/Dennard_scaling)在2005年左右崩溃之后，这一目标变得越来越具有挑战性，这表明顺序计算机的性能不太可能进一步提高。
Consequently, both servers and personal computers have adopted multi-core systems.
因此，服务器和个人计算机都采用了多核系统。
This challenge is compounded by the end of [Moore's Law](https://en.wikipedia.org/wiki/Moore%27s_law), which signifies our diminishing ability to benefit from denser electronic circuits.
这一挑战因[Moore's Law](https://en.wikipedia.org/wiki/Moore%27s_law)的结束而加剧，这标志着我们从更密集的电子电路中获益的能力正在减弱。
It appears that the primary pathway to optimizing performance now lies in specialization, focusing on exploiting the parallelism in workloads.
现在看来，优化性能的主要途径在于专业化，专注于利用工作负载中的并行性。
Due to these technological trends, I foresee that future computers will be massively parallel.
由于这些技术趋势，我预见未来的计算机将是大规模并行的。

However, we are not yet fully equipped for the era of massive parallelism.
然而，我们尚未完全为大规模并行的时代做好准备。
The principal challenge is managing **shared mutable states**, a key aspect of concurrency.
主要挑战是管理**共享可变状态**，这是并发的一个关键方面。
Coordinating multiple cores and resources requires their inputs and outputs to be synchronized through shared mutable states like memory.
协调多个核心和资源需要通过像内存这样的共享可变状态来同步它们的输入和输出。
Yet, managing these states is inherently difficult, both in theory and practice.
然而，管理这些状态在理论上和实践中都是本质上困难的。
For instance, with thousands or millions of cores, how can we efficiently synchronize concurrent access to shared memory?
例如，当拥有成千上万甚至数百万个核心时，我们如何才能高效地同步对共享内存的并发访问？
In the face of nondeterministic thread execution interleaving, how can we ensure the safety of a concurrent program?
在面对非确定性线程执行交错时，我们如何确保并发程序的安全性？
And considering compiler and hardware optimizations, what constitutes the correct specification of a concurrent data structure?
考虑到编译器和硬件优化，什么才构成并发数据结构的正确规范？

Fortunately, in the past ten years, the theory of shared mutable states has made significant advances, greatly facilitating the design and analysis of practical systems utilizing these states.
幸运的是，在过去十年中，共享可变状态理论取得了重大进展，大大便利了利用这些状态的实际系统的设计和分析。
Therefore, in this course, we will explore recent theories of shared mutable states and their application in real-world systems.
因此，在本课程中，我们将探讨共享可变状态的最新理论及其在现实系统中的应用。


### Goal
> 目标

This course is designed for senior undergraduate or graduate students in computer science and related disciplines who have an interest in the contemporary theory and practice of parallel computer systems.
本课程面向计算机科学及相关专业的大四本科生或研究生，他们对现代并行计算机系统的理论与实践感兴趣。
The course aims to equip students with the ability to:
本课程旨在使学生具备以下能力：

- Understand the motivations and challenges of concurrent programming.
  理解并发编程的动机和挑战。
- Learn design patterns and principles for reasoning in concurrent programming.
  学习并发编程中的设计模式和推理原则。
- Design, implement, and evaluate concurrent programs.
  设计、实现和评估并发程序。
- Apply their knowledge to real-world parallel systems.
  将他们的知识应用于现实世界的并行系统。

### Textbook
> 教材

- [Slides](https://docs.google.com/presentation/d/1NMg08N1LUNDPuMxNZ-UMbdH13p8LXgMM3esbWRMowhU/edit?usp=sharing)
  [幻灯片](https://docs.google.com/presentation/d/1NMg08N1LUNDPuMxNZ-UMbdH13p8LXgMM3esbWRMowhU/edit?usp=sharing)
- [Code Documentation](https://kaist-cp.github.io/cs431/cs431/)
  [代码文档](https://kaist-cp.github.io/cs431/cs431/)
- References
  参考资料
    + [The Art of Multiprocessor Programming](https://dl.acm.org/doi/book/10.5555/2385452)
      [多处理器编程的艺术](https://dl.acm.org/doi/book/10.5555/2385452)
    + [The Crossbeam Library Documentation](https://docs.rs/crossbeam/latest/crossbeam/)
      [Crossbeam 库文档](https://docs.rs/crossbeam/latest/crossbeam/)
    + Concurrent reference counting algorithm (TBA)
      并发引用计数算法（待定）
    + [Behaviour-Oriented Concurrency](https://dl.acm.org/doi/10.1145/3622852)
      [面向行为的并发](https://dl.acm.org/doi/10.1145/3622852)
    + [C++ Concurrency in Action](https://www.manning.com/books/c-plus-plus-concurrency-in-action-second-edition)
      [C++ 并发实战](https://www.manning.com/books/c-plus-plus-concurrency-in-action-second-edition)
    + [Rust Atomics and Locks](https://marabos.nl/atomics/)
      [Rust 原子操作和锁](https://marabos.nl/atomics/)

### Prerequisites
> 先修要求

- It is **strongly recommended** that students have completed courses in:
  强烈建议学生已完成以下课程：

    + Mathematics (MAS101): Propositional logic and proof techniques.
      数学 (MAS101)：命题逻辑和证明技巧。
    + Data Structures (CS206): Understanding of linked lists, stacks, and queues.
      数据结构（CS206）：理解链表、栈和队列。
    + Systems Programming (CS230) or Operating Systems (CS330): Familiarity with memory layout, caching, and locking mechanisms.
      系统编程 (CS230) 或操作系统 (CS330)：熟悉内存布局、缓存和锁机制。
    + Programming Principles (CS220) or Programming Languages (CS320): Knowledge of lambda calculus and interpreters.
      编程原理（CS220）或编程语言（CS320）：了解λ演算和解释器。

  A solid foundation in these areas is crucial for success in this course.
  在这些领域的扎实基础对于在本课程中取得成功至关重要。

- Other recommended knowledge that will be beneficial:
  其他推荐的有益知识：

    + Basic understanding of Computer Architecture (CS311).
      计算机体系结构基础知识（CS311）。
    + Programming experience in [Rust](https://www.rust-lang.org/).
      使用 [Rust](https://www.rust-lang.org/) 的编程经验。


### Schedule
> 课程安排

- week 1: CS230/CS330 review on concurrent programming
  第1周：CS230/CS330 并发编程复习
- week 2: Rust
  第2周：Rust
- week 3: lock-based concurrency (API)
  第3周：基于锁的并发（API）
- week 4: lock-based concurrency (implementation 1)
  第4周：基于锁的并发（实现1）
- week 5: lock-based concurrency (implementation 2)
  第5周：基于锁的并发（实现2）
- week 6: lock-based concurrency (application)
  第6周：基于锁的并发（应用）
- week 7: behavior-oriented concurrency (API)
  第7周：面向行为的并发 (API)
- week 8: midterm exam
  第8周：期中考试
- week 9: lock-free concurrency (concept)
  第9周：无锁并发（概念）
- week 10: lock-free concurrency (data structures 1)
  第10周：无锁并发（数据结构1）
- week 11: lock-free concurrency (data structures 2)
  第11周：无锁并发（数据结构2）
- week 12: lock-free concurrency (data structures 3)
  第12周：无锁并发（数据结构3）
- week 13: lock-free concurrency (specification)
  第13周：无锁并发（规范）
- week 14: lock-free concurrency (garbage collection)
  第14周：无锁并发（垃圾回收）
- week 15: behavior-oriented concurrency (implementation)
  第15周：面向行为的并发（实现）
- week 16: final exam
  第16周：期末考试


### Tools
> 工具

Ensure you are proficient with the following development tools:
确保你熟练掌握以下开发工具：

- [Git](https://git-scm.com/): Essential for downloading homework templates and managing your development process.
  [Git](https://git-scm.com/)：下载作业模板和管理您的开发流程的必备工具。
  If you're new to Git, please complete [this tutorial](https://www.atlassian.com/git/tutorials).
  如果你是 Git 新手，请完成 [这个教程](https://www.atlassian.com/git/tutorials)。

    + Follow these steps to set up your repository:
      按照以下步骤设置你的仓库：
        * Clone the upstream repository directly without forking it:
          直接克隆上游仓库而不进行分叉：
          ```bash
          $ git clone --origin upstream git@github.com:kaist-cp/cs431.git
          $ cd cs431
          $ git remote -v
          upstream	git@github.com:kaist-cp/cs431.git (fetch)
          upstream	git@github.com:kaist-cp/cs431.git (push)
          ```
        * To receive updates from the upstream, fetch and merge `upstream/main`:
          要接收来自上游的更新，请获取并合并 `upstream/main`：
          ```bash
          $ git fetch upstream
          $ git merge upstream/main
          ```

    + For managing your development on a Git server, create a private repository:
      要在 Git 服务器上管理您的开发，请创建一个私有仓库：
        * Upgrade to a "PRO" GitHub account, available at no cost.
          升级到“PRO” GitHub 账户，免费提供。
          See the [documentation](https://education.github.com/students).
          查看 [documentation](https://education.github.com/students)。
        * Configure your repository as a remote:
          将你的仓库配置为远程仓库：
          ```bash
          $ git remote add origin git@github.com:<github-id>/cs431.git
          $ git remote -v
          origin	 git@github.com:<github-id>/cs431.git (fetch)
          origin	 git@github.com:<github-id>/cs431.git (push)
          upstream git@github.com:kaist-cp/cs431.git (fetch)
          upstream git@github.com:kaist-cp/cs431.git (push)
          ```
        * Push your work to your repository:
          将你的工作推送到你的仓库：
          ```bash
          $ git push -u origin main
          ```

- [Rust](https://www.rust-lang.org/): The programming language for homework assignments.
  [Rust](https://www.rust-lang.org/)：用于作业 作业的编程语言。
  Rust's ownership type system significantly simplifies the development of large-scale system software.
  Rust 的所有权类型系统显著简化了大型系统软件的开发。

- [ChatGPT](https://chat.openai.com/) or other Large Language Models (LLMs) (optional): Useful for completing your homework.
  [ChatGPT](https://chat.openai.com/) 或其他大型语言模型（LLMs）（可选）：有助于完成你的作业。
    + In an AI-driven era, learning to effectively utilize AI in programming is crucial.
      在人工智能驱动的时代，学习如何在编程中有效使用人工智能至关重要。
      Homework difficulty is adjusted assuming the use of ChatGPT 3.5 or an equivalent tool.
      作业难度是以使用 ChatGPT 3.5 或同等工具为前提进行调整的。

- [Visual Studio Code](https://code.visualstudio.com/) (optional): Recommended for developing your homework, although you may use any editor of your preference.
  [Visual Studio Code](https://code.visualstudio.com/)（可选）：推荐用于开发你的作业，尽管你也可以使用任何你喜欢的编辑器。

- [Single Sign On (SSO)](https://auth.fearless.systems/): Use the following SSO credentials to access [gg](https://gg.kaist.ac.kr) and the [development server](https://cloud.fearless.systems):
  [Single Sign On (SSO)](https://auth.fearless.systems/)：使用以下单点登录凭据访问 [gg](https://gg.kaist.ac.kr) 和 [development server](https://cloud.fearless.systems)：
    + id: KAIST student id (8-digit number)
      ID：KAIST 学生证号（8 位数字）
    + email: KAIST email address (@kaist.ac.kr)
      电子邮件：KAIST 电子邮件地址（@kaist.ac.kr）
    + password: Reset it here: <https://auth.fearless.systems/if/flow/default-recovery-flow/>
      密码：在这里重置它：<https://auth.fearless.systems/if/flow/default-recovery-flow/>
    + Log in to [gg](https://gg.kaist.ac.kr) using the "kaist-cp-class" option, and to the [development server](https://cloud.fearless.systems) using the "OpenID Connect" option.
      使用“kaist-cp-class”选项登录 [gg](https://gg.kaist.ac.kr)，使用“OpenID Connect”选项登录 [development server](https://cloud.fearless.systems)。

- [Development Server](https://cloud.fearless.systems/):
  [Development Server](https://cloud.fearless.systems/)：
    + **IMPORTANT: Do not attempt to hack or overload the server. Please use it responsibly.**
      **重要提示：请勿尝试入侵或使服务器过载。请负责任地使用它。**
    + Create and connect to a workspace to use the terminal or VSCode (after installation).
      创建并连接到工作区以使用终端或 VSCode（安装后）。
    + We recommend using VSCode with the "Rust Analyzer" and "CodeLLDB" plugins.
      我们推荐使用带有“Rust Analyzer”和“CodeLLDB”插件的 VSCode。


## Grading & Honor Code
> 评分与诚信准则

### Cheating
> 作弊

**IMPORTANT: READ CAREFULLY. THIS IS A SERIOUS MATTER.**
**重要：请仔细阅读。这是一个严重的问题。**

- Sign the KAIST CS Honor Code for this semester.
  签署本学期的KAIST计算机科学荣誉守则。
  Failure to do so may lead to expulsion from the course.
  未能做到这一点可能会导致被开除出课程。

- We will employ sophisticated tools to detect code plagiarism.
  我们将使用先进的工具来检测代码抄袭。
    + Search for "code plagiarism detector" on Google Images to understand how these tools can identify advanced forms of plagiarism.
      在谷歌图片上搜索“代码抄袭检测器”，以了解这些工具如何识别高级形式的抄袭。
      Do not attempt plagiarism in any form.
      不要以任何形式进行抄袭。

### Programming Assignments (60%)
> 编程作业（60%）

- All assignments will be announced at the start of the semester.
  所有作业将在学期开始时公布。
- Submit your solutions to <https://gg.kaist.ac.kr/course/19>.
  将您的解决方案提交到 <https://gg.kaist.ac.kr/course/19>。
- Refer to the documentation at <https://kaist-cp.github.io/cs431/cs431_homework/>.
  请参阅位于 <https://kaist-cp.github.io/cs431/cs431_homework/> 的文档。
- You are **permitted** to use ChatGPT or other LLMs.
  你**被允许**使用 ChatGPT 或其他大语言模型。


### Midterm and Final Exams (40%)
> 期中与期末考试（40%）

- Dates & Times: April 15th (Mon), June 10th (Mon), 13:00-15:00
  日期和时间：4月15日（星期一）、6月10日（星期一）、13:00-15:00

- Location: Room 2443, Building E3-1, KAIST
  地点：KAIST E3-1楼2443室

- Physical attendance is required.
  需要亲自出席。
  If necessary, online participation via Zoom will be accommodated.
  如有必要，将提供通过 Zoom 的在线参与方式。

- You are expected to bring your own laptop.
  你需要自带笔记本电脑。
  Laptops can also be borrowed from the School of Computing Administration Team.
  笔记本电脑也可以从计算机学院行政团队借用。

### Attendance (?%)
> 出勤（?%）

- A quiz must be completed on the [Course Management](https://gg.kaist.ac.kr/course/19) website for each session (if any).
  每个会话（如果有的话）都必须在 [Course Management](https://gg.kaist.ac.kr/course/19) 网站上完成一次测验。
  **Quizzes should be completed by the end of the day.**
  **测验应在当天结束前完成。**

- Failing to attend a significant number of sessions will result in an automatic grade of F.
  未能出席大量课程将导致自动获得F等级。


## Communication
> 沟通

### Registration
> 注册

- Ensure your ability to log into the [lab submission website](https://gg.kaist.ac.kr).
  确保您能够登录 [lab submission website](https://gg.kaist.ac.kr)。
    + Use your `kaist-cp-class` account for login.
      使用你的 `kaist-cp-class` 账户登录。
    + Your ID is your `@kaist.ac.kr` email address.
      你的身份证是你的 `@kaist.ac.kr` 邮箱地址。
    + Reset your password here: [https://auth.fearless.systems/if/flow/default-recovery-flow/](https://auth.fearless.systems/if/flow/default-recovery-flow/)
      在此重置您的密码：[https://auth.fearless.systems/if/flow/default-recovery-flow/](https://auth.fearless.systems/if/flow/default-recovery-flow/)
    + Contact the instructor if login issues arise.
      如果出现登录问题，请联系讲师。

### Rules
> 规则

- Course-related announcements and information will be posted on the [course website](https://github.com/kaist-cp/cs431) and the [GitHub issue tracker](https://github.com/kaist-cp/cs431/issues).
  与课程相关的公告和信息将发布在[课程网站](https://github.com/kaist-cp/cs431)和[GitHub issue 跟踪器](https://github.com/kaist-cp/cs431/issues)上。
  It is expected that you read all announcements within 24 hours of their posting.
  预计您在公告发布后的24小时内阅读所有公告。
  Watching the repository is highly recommended for automatic email notifications of new announcements.
  强烈建议关注该仓库，以便自动接收新公告的电子邮件通知。

- Questions about course materials and assignments should be posted in [the course repository's issue tracker](https://github.com/kaist-cp/cs431/issues).
  关于课程资料和作业的问题应发布在 [课程仓库的 issue 跟踪器](https://github.com/kaist-cp/cs431/issues)。
    + Avoid sending emails to the instructor or TAs regarding course materials and assignments.
      避免向教师或助教发送有关课程材料和作业的电子邮件。
    + Research your question using Google and Stack Overflow before posting.
      在发帖之前，先用谷歌和 Stack Overflow 研究你的问题。
    + Describe your question in detail, including:
      详细描述你的问题，包括：
        * Environment (OS, gcc, g++ version, and other relevant program information).
          环境（操作系统、gcc版本以及其他相关程序信息）。
        * Used commands and their results, with logs formatted in code.
          使用过的命令及其结果，日志以代码格式显示。
          See [this guide](https://guides.github.com/features/mastering-markdown/).
          请参见 [this guide](https://guides.github.com/features/mastering-markdown/)。
        * Any changes made to directories or files.
          对目录或文件所做的任何更改。
          For solution files, describe the modified code sections.
          对于解决方案文件，描述修改过的代码部分。
        * Your Google search results, including search terms and learned information.
          您的 Google 搜索结果，包括搜索词和学习到的信息。
    + Use a clear and descriptive title for your issue.
      为您的问题使用一个清晰且具有描述性的标题。
    + For further instructions, read [this section](https://github.com/kaist-cp/cs431#communication) on the course website.
      有关进一步的指示，请在课程网站上阅读 [this section](https://github.com/kaist-cp/cs431#communication)。
    + The requirement to ask questions online first is twofold: It ensures clarity in your query and allows everyone to benefit from shared questions and answers.
      首先在网上提问的要求有两个原因：它确保你的问题清晰，并且让每个人都能从共享的问题和答案中受益。

- Email inquiries should be reserved for confidential or personal matters.
  电子邮件咨询应仅用于保密或个人事务。
  Questions not adhering to this guideline (e.g., course material queries via email) will not be addressed.
  不符合此指南的问题（例如，通过电子邮件提出的课程材料查询）将不予处理。

- Office hours will not cover *new* questions.
  办公时间不涵盖*新*问题。
  Check the issue tracker for similar questions before attending.
  在参加之前，请先查看问题跟踪器中是否有类似的问题。
  If your question is not listed, post it as a new issue for discussion.
  如果您的问题未列出，请将其作为新问题发帖讨论。
  Office hour discussions will focus on unresolved issues.
  办公时间讨论将集中在未解决的问题上。

- Emails to the instructor or head TA should start with "CS431:" in the subject line, followed by a brief description.
  发给讲师或助教组长的电子邮件主题行应以“CS431:”开头，后面跟上简要说明。
  Include your name and student number in the email.
  在电子邮件中包含您的姓名和学号。
  Emails lacking this information (e.g., those without a student number) will not receive a response.
  缺少此信息的电子邮件（例如，没有学号的邮件）将不会收到回复。

- If attending remotely via Zoom (https://kaist.zoom.us/my/jeehoon.kang), set your Zoom name to `<your student number> <your name>` (e.g., `20071163 강지훈`).
  如果通过 Zoom 远程参加（https://kaist.zoom.us/my/jeehoon.kang），请将您的 Zoom 名称设置为 `<your student number> <your name>`（例如，`20071163 강지훈`）。
  Instructions for changing your Zoom name can be found [here](https://support.zoom.us/hc/en-us/articles/201363203-Customizing-your-profile).
  更改您的 Zoom 名称的说明可以在 [这里](https://support.zoom.us/hc/en-us/articles/201363203-Customizing-your-profile) 找到。

- The course is conducted in English.
  这门课程以英语进行。
  However, you may ask questions in Korean, which will be translated into English.
  但是，您可以用韩语提问，这些问题将被翻译成英语。

## Ignore
> 忽略

1830eaed90e5986c75320daaf131bd3730b8575e866c4e92935a690e7c2a0717

## Repository Reading Order / 仓库阅读顺序

### Main Entry Points / 总入口

1. `README.md`: course overview, schedule, tools, grading, and communication rules. / 课程总说明、周次安排、工具、评分规则和沟通方式。
2. `homework/README.md`: general homework workflow, test commands, sanitizer usage, and grading-script notes. / 作业通用说明，包括测试命令、sanitizer 使用方式和评分脚本注意事项。
3. `homework/doc/*.md`: detailed instructions for each homework assignment. / 每个作业的具体说明。

### Recommended Homework Order / 推荐作业顺序

1. `homework/doc/hello_server.md`: parallel web server, thread pool, cache, and TCP basics. / 并行 Web 服务器、线程池、缓存和 TCP 基础。
2. `homework/doc/linked_list.md`: unsafe Rust and raw pointer practice. / unsafe Rust 和裸指针练习。
3. `homework/doc/list_set.md`: fine-grained lock-coupling set based on linked lists; it builds on the unsafe pointer experience from `linked_list`. / 基于链表的细粒度锁耦合集合，依赖 `linked_list` 中的 unsafe 指针经验。
4. `homework/doc/arc.md`: atomic reference counting and synchronization. / 原子引用计数和同步。
5. `homework/doc/hash_table.md`: lock-free hash table; implement `growable_array.rs` before `split_ordered_list.rs`. / 无锁哈希表；先实现 `growable_array.rs`，再实现 `split_ordered_list.rs`。
6. `homework/doc/hazard_pointer.md`: hazard pointers for deferred memory reclamation. / 用于延迟内存回收的 Hazard Pointer。
7. `homework/doc/boc.md`: Behaviour-Oriented Concurrency runtime, conceptually later and more independent. / 面向行为并发运行时，概念上更靠后，也相对独立。
