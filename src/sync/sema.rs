use alloc::collections::btree_map::BTreeMap;
use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Arc;
use sync::Mutex;
use sync::Spin;

use crate::sbi;
use crate::thread::{self, Thread};

/// Atomic counting semaphore
///
/// # Examples
/// ```
/// let sema = Semaphore::new(0);
/// sema.down();
/// sema.up();
/// ```
pub struct Semaphore {
    inner: Mutex<SemaInner, Spin>,
}

struct SemaInner {
    value: usize,
    waiters: BTreeMap<u32, VecDeque<Arc<Thread>>>,
}

impl Semaphore {
    /// Creates a new semaphore of initial value n.
    pub fn new(n: usize) -> Self {
        Self {
            inner: Mutex::new(SemaInner {
                value: n,
                waiters: BTreeMap::new(),
            }),
        }
    }

    /// P operation
    pub fn down(&self) {
        let old = sbi::interrupt::set(false);

        // Is semaphore available?
        while self.value() == 0 {
            let priority = thread::get_priority();
            self.inner
                .lock()
                .waiters
                .entry(priority)
                .or_insert_with(VecDeque::new)
                .push_back(thread::current());
            // Block the current thread until it's awakened by an `up` operation
            thread::block();
        }

        self.inner.lock().value -= 1;

        sbi::interrupt::set(old);
    }

    /// V operation
    pub fn up(&self) {
        let old = sbi::interrupt::set(false);
        let mut guard = self.inner.lock();
        let count = guard.value;
        guard.value += 1;

        // Check if we need to wake up a sleeping waiter (highest priority first)
        let thread_to_wake = if let Some(&priority) = guard.waiters.keys().next_back() {
            assert_eq!(count, 0);

            let thread = guard
                .waiters
                .get_mut(&priority)
                .and_then(|deque| deque.pop_front());

            // Remove the priority entry if the deque is now empty
            if guard.waiters.get(&priority).map_or(false, |d| d.is_empty()) {
                guard.waiters.remove(&priority);
            }

            thread
        } else {
            None
        };

        guard.release();

        if let Some(t) = thread_to_wake {
            thread::wake_up(t);
        }

        sbi::interrupt::set(old);
    }

    /// Get the current value of a semaphore
    pub fn value(&self) -> usize {
        self.inner.lock().value
    }
}
