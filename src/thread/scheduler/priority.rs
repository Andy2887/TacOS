use alloc::collections::VecDeque;
use alloc::sync::Arc;

use crate::thread::{BTreeMap, Schedule, Thread};

/// (lab1) Priority Scheduler
#[derive(Default)]
pub struct Priority {
    priority_to_thread: BTreeMap<u32, VecDeque<Arc<Thread>>>,
    thread_to_priority: BTreeMap<isize, u32>, // for O(log n) lookup of current bucket
}

impl Schedule for Priority {
    fn register(&mut self, thread: Arc<Thread>) {
        use core::sync::atomic::Ordering;
        let priority = thread.priority.load(Ordering::Relaxed);
        let tid = thread.id();
        self.priority_to_thread
            .entry(priority)
            .or_default()
            .push_back(thread);
        self.thread_to_priority.insert(tid, priority);
    }

    fn schedule(&mut self) -> Option<Arc<Thread>> {
        // Find the highest priority bucket
        if let Some(bucket) = self.priority_to_thread.pop_last() {
            let (priority, mut thread_deque) = bucket;
            let thread = thread_deque.pop_front();
            if thread.is_some() {
                self.thread_to_priority
                    .remove(&thread.as_ref().unwrap().id());
            }
            if thread_deque.len() > 0 {
                self.priority_to_thread.insert(priority, thread_deque);
            }

            thread
        } else {
            None
        }
    }
}
