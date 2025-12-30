use alloc::collections::VecDeque;
use alloc::sync::Arc;

use crate::thread::{BTreeMap, Schedule, Thread, PRI_MIN};

/// (lab1) Priority Scheduler
#[derive(Default)]
pub struct Priority {
    priority_to_thread: BTreeMap<u32, VecDeque<Arc<Thread>>>,
    thread_to_priority: BTreeMap<isize, u32>, // for O(log n) lookup of current bucket
}

impl Schedule for Priority {
    fn register(&mut self, thread: Arc<Thread>) {
        let priority = thread.effective_priority();
        let tid = thread.id();

        #[cfg(feature = "debug")]
        kprintln!("[DEBUG register] tid: {}, priority: {}", tid, priority);

        #[cfg(feature = "debug")]
        {
            kprintln!("[DEBUG register] Other threads in scheduler:");
            for (pri, threads) in self.priority_to_thread.iter().rev() {
                for t in threads.iter() {
                    kprintln!("[DEBUG register]   tid: {}, priority: {}", t.id(), pri);
                }
            }
        }

        self.priority_to_thread
            .entry(priority)
            .or_default()
            .push_back(thread);
        self.thread_to_priority.insert(tid, priority);
    }

    fn schedule(&mut self) -> Option<Arc<Thread>> {
        #[cfg(feature = "debug")]
        {
            kprintln!("[DEBUG schedule] Threads in scheduler (sorted by priority):");
            for (pri, threads) in self.priority_to_thread.iter().rev() {
                for t in threads.iter() {
                    kprintln!("[DEBUG schedule]   tid: {}, priority: {}", t.id(), pri);
                }
            }
        }

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

            #[cfg(feature = "debug")]
            if let Some(ref t) = thread {
                kprintln!(
                    "[DEBUG schedule] Chosen thread - tid: {}, priority: {}",
                    t.id(),
                    priority
                );
            }

            thread
        } else {
            None
        }
    }

    fn change_priority(&mut self, thread: Arc<Thread>, priority: u32) {
        let tid = thread.id();
        if let Some(old_priority) = self.thread_to_priority.get(&tid).copied() {
            #[cfg(feature = "debug")]
            kprintln!(
                "[DEBUG change_priority] BEFORE - tid: {}, priority: {}",
                tid,
                old_priority
            );

            self.thread_to_priority
                .entry(tid)
                .and_modify(|v| *v = priority);

            let old_bucket = self.priority_to_thread.get_mut(&old_priority).unwrap();
            if let Some(index) = old_bucket.iter().position(|x| Arc::ptr_eq(x, &thread)) {
                old_bucket.remove(index);

                if old_bucket.is_empty() {
                    self.priority_to_thread.remove(&old_priority);
                }
            }

            self.priority_to_thread
                .entry(priority)
                .or_default()
                .push_back(thread);

            #[cfg(feature = "debug")]
            kprintln!(
                "[DEBUG change_priority] AFTER - tid: {}, priority: {}",
                tid,
                priority
            );
        }
    }

    fn highest_priority(&mut self) -> u32 {
        if let Some(highest_priority) = self.priority_to_thread.keys().next_back() {
            *highest_priority
        } else {
            PRI_MIN
        }
    }
}
