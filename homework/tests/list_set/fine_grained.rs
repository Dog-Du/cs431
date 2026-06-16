use std::collections::HashSet;
use std::iter::zip;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::{Acquire, Release};
use std::thread;

use cs431_homework::test::adt::set;
use cs431_homework::{ConcurrentSet, FineGrainedListSet};
use rand::prelude::*;

#[test]
fn smoke() {
    let set = FineGrainedListSet::new();
    assert!(set.insert(1));
    assert!(set.insert(2));
    assert!(set.insert(3));
    assert!(set.remove(&2));
    for (r, v) in zip(set.iter(), [1, 3]) {
        assert_eq!(*r, v);
    }
    assert!(set.remove(&3));
}

#[test]
fn stress_sequential() {
    const STEPS: usize = 4096;
    set::stress_sequential::<_, FineGrainedListSet<u8>>(STEPS);
}

#[test]
fn stress_concurrent() {
    const THREADS: usize = 16;
    const STEPS: usize = 4096 * 16;
    set::stress_concurrent::<_, FineGrainedListSet<u8>>(THREADS, STEPS);
}

#[test]
fn log_concurrent() {
    const THREADS: usize = 16;
    const STEPS: usize = 4096 * 16;
    set::log_concurrent::<_, FineGrainedListSet<u8>>(THREADS, STEPS);
}

/// Check the consistency of iterator while other operations are running concurrently.
/// 在其他操作并发运行时，检查迭代器的一致性。
#[test]
fn iter_consistent() {
    const THREADS: usize = 15;
    const STEPS: usize = 4096 * 16;

    let set = FineGrainedListSet::new();

    // pre-fill with even numbers
    // 预先填充偶数
    for i in (0..100).step_by(2).rev() {
        assert!(set.insert(i));
    }
    let evens = set.iter().copied().collect::<HashSet<_>>();

    let done = AtomicBool::new(false);
    thread::scope(|s| {
        // Ensure handles lives to the end.
        // 确保手柄始终有效。
        let mut handles = Vec::with_capacity(THREADS + 1);
        // insert or remove odd numbers
        // 插入或删除奇数
        for _ in 0..THREADS {
            handles.push(s.spawn(|| {
                let mut rng = rand::rng();
                for _ in 0..STEPS {
                    let key = 2 * rng.random_range(0..50) + 1;
                    if rng.random() {
                        set.insert(key);
                    } else {
                        set.remove(&key);
                    }
                }
                done.store(true, Release);
            }));
        }
        handles.push(s.spawn(|| {
            while !done.load(Acquire) {
                let snapshot = set.iter().copied().collect::<Vec<_>>();
                // sorted
                // 已排序
                assert!(snapshot.windows(2).all(|k| k[0] <= k[1]));
                // even numbers are not touched
                // 偶数不受影响
                let snapshot = snapshot.into_iter().collect::<HashSet<_>>();
                assert!(evens.is_subset(&snapshot));
            }
        }));
    });
}
