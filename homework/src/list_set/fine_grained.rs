use std::cmp::Ordering::{self, *};
use std::ops::Deref;
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
        let mut node = *self.0;
        while !node.is_null() {
            let ord = unsafe { &(*node).data }.cmp(key);

            if ord == Ordering::Less {
                let next_guard = unsafe { &(*node).next }.lock().unwrap();
                self.0 = next_guard;
                node = *self.0;
            } else if ord == Ordering::Equal {
                return true;
            } else {
                return false;
            }
        }
        false
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
        let mut cursor = Cursor(self.head.lock().unwrap());
        let found = cursor.find(key);
        (found, cursor)
    }
}

impl<T: Ord> ConcurrentSet<T> for FineGrainedListSet<T> {
    fn contains(&self, key: &T) -> bool {
        self.find(key).0
    }

    fn insert(&self, key: T) -> bool {
        let (found, cursor) = self.find(&key);
        if found {
            return false;
        }

        let mut curr = cursor.0;
        let next = *curr;
        let node = Node::new(key, next);
        *curr = node;
        true
    }

    fn remove(&self, key: &T) -> bool {
        let (found, cursor) = self.find(key);
        if !found {
            return false;
        }

        let mut curr = cursor.0;
        let next_guard = unsafe { &(**curr).next }.lock().unwrap();
        let gc_node = *curr;
        *curr = *next_guard;
        drop(next_guard);
        let _ = unsafe { Box::from_raw(gc_node) };
        true
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
        if (*self.cursor).is_null() {
            return None;
        }
        let node = *self.cursor;
        self.cursor = unsafe { (*node).next.lock().unwrap() };
        Some(unsafe { &(*node).data })
    }
}

impl<T> Drop for FineGrainedListSet<T> {
    fn drop(&mut self) {
        let mut node = self.head.lock().unwrap();
        while !(*node).is_null() {
            let next_guard = unsafe { &(**node).next }.lock().unwrap();
            let gc_node = *node;
            *node = *next_guard;
            drop(next_guard);
            let _ = unsafe { Box::from_raw(gc_node) };
        }
        drop(node);
    }
}

impl<T> Default for FineGrainedListSet<T> {
    fn default() -> Self {
        Self::new()
    }
}
