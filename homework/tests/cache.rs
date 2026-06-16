use std::sync::Barrier;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread::scope;
use std::time::Duration;

use crossbeam_channel::bounded;
use cs431_homework::hello_server::Cache;

const NUM_THREADS: usize = 8;
const NUM_KEYS: usize = 128;

#[test]
fn cache_no_duplicate_sequential() {
    let cache = Cache::default();
    assert_eq!(cache.get_or_insert_with(1, |_| 1), 1);
    assert_eq!(cache.get_or_insert_with(2, |_| 2), 2);
    assert_eq!(cache.get_or_insert_with(3, |_| 3), 3);
    assert_eq!(cache.get_or_insert_with(1, |_| panic!()), 1);
    assert_eq!(cache.get_or_insert_with(2, |_| panic!()), 2);
    assert_eq!(cache.get_or_insert_with(3, |_| panic!()), 3);
}

#[test]
fn cache_no_duplicate_concurrent() {
    for _ in 0..8 {
        let cache = Cache::default();
        let barrier = Barrier::new(NUM_THREADS);
        // Count the number of times the computation is run.
        // 计算运行的次数。
        let num_compute = AtomicUsize::new(0);
        scope(|s| {
            for _ in 0..NUM_THREADS {
                let _ = s.spawn(|| {
                    let _ = barrier.wait();
                    for key in 0..NUM_KEYS {
                        let _ = cache.get_or_insert_with(key, |k| {
                            let _ = num_compute.fetch_add(1, Ordering::Relaxed);
                            k
                        });
                    }
                });
            }
        });
        assert_eq!(num_compute.load(Ordering::Relaxed), NUM_KEYS);
    }
}

#[test]
fn cache_no_block_disjoint() {
    let cache = &Cache::default();

    scope(|s| {
        // T1 blocks while inserting 1.
        // T1 在插入 1 时被阻塞。
        let (t1_quit_sender, t1_quit_receiver) = bounded(0);
        let _ = s.spawn(move || {
            let _ = cache.get_or_insert_with(1, |k| {
                // block T1
                // T1 块
                t1_quit_receiver.recv().unwrap();
                k
            });
        });

        // T2 must not be blocked by T1 when inserting 2.
        // 在插入 2 时，T1 不得阻塞 T2。
        let (t2_done_sender, t2_done_receiver) = bounded(0);
        let _ = s.spawn(move || {
            let _ = cache.get_or_insert_with(2, |k| k);
            t2_done_sender.send(()).unwrap();
        });

        // If T2 is blocked, then this will time out.
        // 如果 T2 被阻塞，那么这将超时。
        t2_done_receiver
            .recv_timeout(Duration::from_secs(3))
            .expect("Inserting a different key should not block");

        // clean up
        // 打扫
        t1_quit_sender.send(()).unwrap();
    });
}

#[test]
fn cache_no_reader_block() {
    let cache = &Cache::default();

    scope(|s| {
        let (t1_quit_sender, t1_quit_receiver) = bounded(0);
        let (t3_done_sender, t3_done_receiver) = bounded(0);

        // T1 blocks while inserting 1.
        // T1 在插入 1 时被阻塞。
        let _ = s.spawn(move || {
            let _ = cache.get_or_insert_with(1, |k| {
                // T2 is blocked by T1 when reading 1
                // 当 T2 读取 1 时被 T1 阻塞
                let _ = s.spawn(move || cache.get_or_insert_with(1, |_| panic!()));

                // T3 should not be blocked when inserting 3.
                // 在插入3时不应阻塞T3。
                let _ = s.spawn(move || {
                    let _ = cache.get_or_insert_with(3, |k| k);
                    t3_done_sender.send(()).unwrap();
                });

                // block T1
                // T1 块
                t1_quit_receiver.recv().unwrap();
                k
            });
        });

        // If T3 is blocked, then this will time out.
        // 如果 T3 被阻塞，那么这将超时。
        t3_done_receiver
            .recv_timeout(Duration::from_secs(3))
            .expect("Inserting a different key should not block");

        // clean up
        // 打扫
        t1_quit_sender.send(()).unwrap();
    });
}
