use std::cmp::Ordering::*;
use std::sync::{Mutex, MutexGuard};
use std::{mem, ptr};

use crate::ConcurrentSet;

#[derive(Debug)]
struct Node<T> {
    data: T,
    next: Mutex<*mut Node<T>>,
}

/// Concurrent sorted singly linked list using fine-grained lock-coupling.
/// 使用细粒度锁耦合的并发排序单向链表。
#[derive(Debug)]
pub struct FineGrainedListSet<T> {
    head: Mutex<*mut Node<T>>,
}

unsafe impl<T: Send> Send for FineGrainedListSet<T> {}
unsafe impl<T: Send> Sync for FineGrainedListSet<T> {}

/// Reference to the `next` field of previous node which points to the current node.
/// 引用前一个节点的 `next` 字段，该字段指向当前节点。
///
/// For example, given the following linked list:
/// 例如，给定以下链表：
///
/// ```text
/// head -> 1 -> 2 -> 3 -> null
/// ```
///
/// If `cursor` is currently at node 2, then `cursor.0` should be the `MutexGuard` obtained from the
/// 如果 `cursor` 当前位于节点 2，那么 `cursor.0` 应该是从中获得的 `MutexGuard`
/// `next` of node 1. In particular, `cursor.0.as_ref().unwrap()` creates a shared reference to node
/// 节点 1 的 `next`。特别是，`cursor.0.as_ref().unwrap()` 创建了指向节点的共享引用
/// 2.
struct Cursor<'l, T>(MutexGuard<'l, *mut Node<T>>);

impl<T> Node<T> {
    fn new(data: T, next: *mut Self) -> *mut Self {
        Box::into_raw(Box::new(Self {
            data,
            next: Mutex::new(next),
        }))
    }
}

impl<T: Ord> Cursor<'_, T> {
    /// Moves the cursor to the position of key in the sorted list.
    /// 将cursor移动到排序列表中键的位置。
    /// Returns whether the value was found.
    /// 返回是否找到该值。
    fn find(&mut self, key: &T) -> bool {
        todo!()
    }
}

impl<T> FineGrainedListSet<T> {
    /// Creates a new list.
    /// 创建一个新的列表。
    pub fn new() -> Self {
        Self {
            head: Mutex::new(ptr::null_mut()),
        }
    }
}

impl<T: Ord> FineGrainedListSet<T> {
    fn find(&self, key: &T) -> (bool, Cursor<'_, T>) {
        todo!()
    }
}

impl<T: Ord> ConcurrentSet<T> for FineGrainedListSet<T> {
    fn contains(&self, key: &T) -> bool {
        self.find(key).0
    }

    fn insert(&self, key: T) -> bool {
        todo!()
    }

    fn remove(&self, key: &T) -> bool {
        todo!()
    }
}

#[derive(Debug)]
pub struct Iter<'l, T> {
    cursor: MutexGuard<'l, *mut Node<T>>,
}

impl<T> FineGrainedListSet<T> {
    /// An iterator visiting all elements.
    /// 一个访问所有元素的迭代器。
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            cursor: self.head.lock().unwrap(),
        }
    }
}

impl<'l, T> Iterator for Iter<'l, T> {
    type Item = &'l T;

    fn next(&mut self) -> Option<Self::Item> {
        todo!()
    }
}

impl<T> Drop for FineGrainedListSet<T> {
    fn drop(&mut self) {
        todo!()
    }
}

impl<T> Default for FineGrainedListSet<T> {
    fn default() -> Self {
        Self::new()
    }
}
