//! Thread pool that joins all thread when dropped.
//! 线程池在被丢弃时会等待所有线程完成。

use std::ops::{Add, Sub};
// NOTE: Crossbeam channels are MPMC, which means that you don't need to wrap the receiver in
// 注意：Crossbeam 通道是 MPMC，这意味着你不需要将接收器包装在
// Arc<Mutex<..>>. Just clone the receiver and give it to each worker thread.
// Arc<Mutex<..>>。只需克隆接收器并把它分配给每个工作线程。
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use crossbeam_channel::{Sender, unbounded};
use crossbeam_epoch::Pointable;

struct Job(Box<dyn FnOnce() + Send + 'static>);

#[derive(Debug)]
struct Worker {
    _id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Drop for Worker {
    /// When dropped, the thread's `JoinHandle` must be `join`ed.  If the worker panics, then this
    /// 当掉落时，线程的 `JoinHandle` 必须被 `join`。如果工人惊慌，那么这个
    /// function should panic too.
    /// 函数也应该panic。
    ///
    /// NOTE: The thread is detached if not `join`ed explicitly.
    /// 注意：如果没有被明确 `join`，线程将被分离。
    fn drop(&mut self) {
        if let Some(t) = self.thread.take() {
            t.join().unwrap()
        }
    }
}

/// Internal data structure for tracking the current job status. This is shared by worker closures
/// 用于跟踪当前作业状态的内部数据结构。这由工作者闭包共享
/// via `Arc` so that the workers can report to the pool that it started/finished a job.
/// 通过 `Arc`，以便工人可以向池报告它已开始/完成一项工作。
#[derive(Debug, Default)]
struct ThreadPoolInner {
    job_count: Mutex<usize>,
    empty_condvar: Condvar,
}

impl ThreadPoolInner {
    /// Increment the job count.
    /// 增加工作数量。
    fn start_job(&self) {
        let _ = self.job_count.lock().unwrap().add(1);
    }

    /// Decrement the job count.
    /// 减少作业数量。
    fn finish_job(&self) {
        let _ = self.job_count.lock().unwrap().sub(1);
        self.empty_condvar.notify_one();
    }

    /// Wait until the job count becomes 0.
    /// 等待直到作业数量变为0。
    ///
    /// NOTE: We can optimize this function by adding another field to `ThreadPoolInner`, but let's
    /// 注意：我们可以通过向 `ThreadPoolInner` 添加另一个字段来优化此函数，但让我们
    /// not care about that in this homework.
    /// 在这个作业中不在乎那个。
    fn wait_empty(&self) {
        loop {
            let guard = self
                .empty_condvar
                .wait(self.job_count.lock().unwrap())
                .unwrap();
            if *guard == 0 {
                break;
            }
        }
    }
}

/// Thread pool.
/// 线程池
#[derive(Debug)]
pub struct ThreadPool {
    _workers: Vec<Worker>,
    job_sender: Option<Sender<Job>>,
    pool_inner: Arc<ThreadPoolInner>,
}

impl ThreadPool {
    /// Create a new ThreadPool with `size` threads.
    /// 使用 `size` 个线程创建一个新的线程池。
    ///
    /// # Panics
    /// # panic
    ///
    /// Panics if `size` is 0.
    /// 如果 `size` 为 0，则会引发panic。
    pub fn new(size: usize) -> Self {
        assert!(size > 0);
        let (sender, receiver) = unbounded();
        let mut ret = Self {
            _workers: Vec::new(),
            job_sender: Some(sender),
            pool_inner: Arc::new(ThreadPoolInner::default()),
        };

        for i in 0..size {
            let receiver = receiver.clone();
            ret._workers.push(Worker {
                _id: i,
                thread: Some(thread::spawn(move || -> () {
                    for job in receiver.into_iter() {
                        let Job(f) = job;
                        f();
                    }
                    ()
                })),
            });
        }

        ret
    }

    /// Execute a new job in the thread pool.
    /// 在线程池中执行一个新任务。
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.job_sender
            .as_ref()
            .unwrap()
            .send(Job(Box::new(f)))
            .unwrap();
    }

    /// Block the current thread until all jobs in the pool have been executed.
    /// 阻塞当前线程，直到池中的所有任务都被执行完毕。
    ///
    /// NOTE: This method has nothing to do with `JoinHandle::join`.
    /// 注意：此方法与 `JoinHandle::join` 无关。
    pub fn join(&self) {
        self.pool_inner.wait_empty();
    }
}

impl Drop for ThreadPool {
    /// When dropped, all worker threads' `JoinHandle` must be `join`ed. If the thread panicked,
    /// 当被丢弃时，所有工作线程的 `JoinHandle` 必须被 `join`。如果线程发生panic，
    /// then this function should panic too.
    /// 那么这个函数也应该发生panic。
    fn drop(&mut self) {
        self._workers.drain(..);
    }
}
