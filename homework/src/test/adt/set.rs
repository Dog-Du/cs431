//! Testing utilities for set types.
//! 用于集合类型的测试工具。

use core::fmt::Debug;
use core::hash::Hash;

use crossbeam_epoch::Guard;

use super::map;
use crate::test::RandGen;
use crate::{ConcurrentMap, ConcurrentSet};

// A set can be seen as a map with value `()`. Thus, we can reuse the tests for maps.
// 一个集合可以被看作是一个值为 `()` 的映射。因此，我们可以重用映射的测试。
impl<T, S: ConcurrentSet<T>> ConcurrentMap<T, ()> for S {
    fn lookup<'a>(&'a self, key: &T, _guard: &'a Guard) -> Option<&'a ()> {
        if self.contains(key) { Some(&()) } else { None }
    }

    fn insert(&self, key: T, _value: (), _guard: &Guard) -> Result<(), ()> {
        if self.insert(key) { Ok(()) } else { Err(()) }
    }

    fn delete<'a>(&'a self, key: &T, _guard: &'a Guard) -> Result<&'a (), ()> {
        if self.remove(key) { Ok(&()) } else { Err(()) }
    }
}

/// See `map::stress_sequential`.
/// 请参见 `map::stress_sequential`。
pub fn stress_sequential<T: Debug + Clone + Eq + Hash + RandGen, S: Default + ConcurrentSet<T>>(
    steps: usize,
) {
    map::stress_sequential::<T, (), S>(steps);
}

/// See `map::stress_concurrent`.
/// 请参见 `map::stress_concurrent`。
pub fn stress_concurrent<T: Debug + Eq + RandGen, S: Default + Sync + ConcurrentSet<T>>(
    threads: usize,
    steps: usize,
) {
    map::stress_concurrent::<T, (), S>(threads, steps);
}

/// See `map::log_concurrent`.
/// 请参见 `map::log_concurrent`。
pub fn log_concurrent<
    T: Clone + Debug + Eq + Hash + RandGen + Send,
    S: Default + Sync + ConcurrentSet<T>,
>(
    threads: usize,
    steps: usize,
) {
    map::log_concurrent::<T, (), S>(threads, steps);
}
