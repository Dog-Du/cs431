//! TcpListener that can be cancelled.
//! 可以取消的 TcpListener。

use std::io;
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::atomic::{AtomicBool, Ordering};

/// Like `std::net::tcp::TcpListener`, but `cancel`lable.
/// 像 `std::net::tcp::TcpListener`，但 `cancel` 可标记。
#[derive(Debug)]
pub struct CancellableTcpListener {
    inner: TcpListener,

    /// An atomic boolean flag that indicates if the listener is `cancel`led.
    /// 一个原子布尔标志，指示监听器是否被 `cancel` 化。
    ///
    /// NOTE: This can be safely read/written by multiple thread at the same time (note that its
    /// 注意：这个可以被多个线程同时安全地读/写（注意它的
    /// methods take `&self` instead of `&mut self`). To set the flag, use `store` method with
    /// 方法使用 `&self` 而不是 `&mut self`)。要设置标志，请使用 `store` 方法
    /// `Ordering::Release`. To read the flag, use `load` method with `Ordering::Acquire`. We  will
    /// `Ordering::Release`。要读取旗帜，请使用 `load` 方法和 `Ordering::Acquire`。我们将
    /// discuss their precise semantics later.
    /// 稍后讨论它们的精确语义。
    is_canceled: AtomicBool,
}

/// Like `std::net::tcp::Incoming`, but stops `accept`ing connections if the listener is `cancel`ed.
/// 像 `std::net::tcp::Incoming`，但如果监听器被 `cancel`，它会停止 `accept` 连接。
#[derive(Debug)]
pub struct Incoming<'a> {
    listener: &'a CancellableTcpListener,
}

impl CancellableTcpListener {
    /// Wraps `TcpListener::bind`.
    /// 包装 `TcpListener::bind`。
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<CancellableTcpListener> {
        let listener = TcpListener::bind(addr)?;
        Ok(CancellableTcpListener {
            inner: listener,
            is_canceled: AtomicBool::new(false),
        })
    }

    /// Signals the listener to stop accepting new connections.
    /// 通知监听器停止接受新的连接。
    pub fn cancel(&self) -> io::Result<()> {
        // Set the flag first and make a bogus connection to itself to wake up the listener blocked
        // 先设置标志，然后与自身建立一个假的连接以唤醒被阻塞的监听器
        // in `accept`. Use `TcpListener::local_addr` and `TcpStream::connect`.
        // 在 `accept` 中。使用 `TcpListener::local_addr` 和 `TcpStream::connect`。
        self.is_canceled.store(true, Ordering::Release);
        TcpStream::connect(self.inner.local_addr()?).map(|_| ())
    }

    /// Returns an iterator over the connections being received on this listener. The returned
    /// 返回一个迭代器，遍历正在此监听器上接收的连接。返回的
    /// iterator will return `None` if the listener is `cancel`led.
    /// 迭代器如果监听器被 `cancel` 了，将返回 `None`。
    pub fn incoming(&self) -> Incoming<'_> {
        Incoming { listener: self }
    }
}

impl Iterator for Incoming<'_> {
    type Item = io::Result<TcpStream>;
    /// Returns None if the listener is `cancel()`led.
    /// 如果监听器被 `cancel()` 屏蔽，则返回 None。
    fn next(&mut self) -> Option<Self::Item> {
        if self.listener.is_canceled.load(Ordering::Acquire) {
            return None;
        }

        let stream = self.listener.inner.accept().map(|p| p.0);

        if self.listener.is_canceled.load(Ordering::Acquire) {
            None
        } else {
            Some(stream)
        }
    }
}
