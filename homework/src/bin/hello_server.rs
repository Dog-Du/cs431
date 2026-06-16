use std::io;
use std::sync::Arc;
use std::sync::mpsc::{channel, sync_channel};

use cs431_homework::hello_server::{CancellableTcpListener, Handler, Statistics, ThreadPool};

const ADDR: &str = "localhost:7878";

fn main() -> io::Result<()> {
    // Use a browser that doesn't cache too eagerly so that request is always sent. For example,
    // 使用不会过早缓存的浏览器，以确保请求始终被发送。例如，
    // Firefox works well.  If you want to test using command line only, use curl. If you want to
    // Firefox 运行良好。如果你只想使用命令行进行测试，请使用 curl。如果你想
    // run it on the lab server, you may need to change the port number to something else.
    // 在实验室服务器上运行它，你可能需要将端口号改为其他的。
    println!("Run `curl http://{ADDR}/KEY` to query the server with KEY");

    // The thread pool.
    // The 线程池.
    //
    // In the thread pool, we'll execute:
    // 在线程池中，我们将执行：
    //
    // - A listener: it accepts incoming connections, and creates a new worker for each connection.
    // - 监听器：它接受传入连接，并为每个连接创建一个新的工作线程。
    //
    // - Workers (once for each incoming connection): a worker handles an incoming connection and
    // - 工作线程（每个传入连接一次）：一个工作线程处理一个传入连接并
    //   sends a corresponding report to the reporter.
    // 向报告人发送相应的报告。
    //
    // - A reporter: it aggregates the reports from the workers and processes the statistics. When
    // - 一名记者：它汇总工人的报告并处理统计数据。 当
    //   it ends, it sends the statistics to the main thread.
    // 它结束时，会将统计数据发送到主线程。
    let pool = Arc::new(ThreadPool::new(7));

    // The (MPSC) channel of reports between workers and the reporter.
    // 工人和报告者之间的（MPSC）报告通道。
    let (report_sender, report_receiver) = channel();

    // The (SPSC one-shot) channel of stats between the reporter and the main thread.
    // 报告者和主线程之间的（SPSC 一次性）统计通道。
    let (stat_sender, stat_receiver) = sync_channel(0);

    // Listens to the address.
    // 听演讲。
    let listener = Arc::new(CancellableTcpListener::bind(ADDR)?);

    // Installs a Ctrl-C handler.
    // 安装一个 Ctrl-C 处理程序。
    let ctrlc_listener_handle = listener.clone();
    ctrlc::set_handler(move || {
        ctrlc_listener_handle.cancel().unwrap();
    })
    .expect("Error setting Ctrl-C handler");

    // Executes the listener.
    // 执行监听器。
    let listener_pool = pool.clone();
    pool.execute(move || {
        // Creates the request handler.
        // 创建请求处理程序。
        let handler = Handler::default();

        // For each incoming connection...
        // 对于每一个传入的连接...
        for (id, stream) in listener.incoming().enumerate() {
            // send a job to the thread pool.
            // 将一个任务发送到线程池。
            let report_sender = report_sender.clone();
            let handler = handler.clone();
            listener_pool.execute(move || {
                let report = handler.handle_conn(id, stream.unwrap());
                report_sender.send(report).unwrap();
            });
        }
    });

    // Executes the reporter.
    // 处决记者。
    pool.execute(move || {
        let mut stats = Statistics::default();
        for report in report_receiver {
            println!("[report] {report:?}");
            stats.add_report(report);
        }

        println!("[sending stat]");
        stat_sender.send(stats).unwrap();
        println!("[sent stat]");
    });

    // Blocks until the reporter sends the statistics.
    // 阻塞，直到报告器发送统计信息。
    let stat = stat_receiver.recv().unwrap();
    println!("[stat] {stat:?}");

    Ok(())
    // When the pool is dropped, all worker threads are joined.
    // 当池被释放时，所有工作线程都会被加入。
}
