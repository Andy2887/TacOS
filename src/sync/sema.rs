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
    waiters: VecDeque<Arc<Thread>>,
}

impl Semaphore {
    /// Creates a new semaphore of initial value n.
    pub fn new(n: usize) -> Self {
        Self {
            inner: Mutex::new(SemaInner {
                value: n,
                waiters: VecDeque::new(),
            }),
        }
    }

    /// P operation
    pub fn down(&self) {
        let old = sbi::interrupt::set(false);

        // Is semaphore available?
        while self.value() == 0 {
            self.inner.lock().waiters.push_back(thread::current());
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

        // Check if we need to wake up a sleeping waiter (highest effective priority first)
        let thread_to_wake = if !guard.waiters.is_empty() {
            assert_eq!(count, 0);

            // Find the index of the thread with the highest effective priority
            let mut max_priority = 0;
            let mut max_index = 0;

            for (index, thread) in guard.waiters.iter().enumerate() {
                let eff_priority = thread.effective_priority();
                if index == 0 || eff_priority > max_priority {
                    max_priority = eff_priority;
                    max_index = index;
                }
            }

            // Remove and return the thread with the highest effective priority
            guard.waiters.remove(max_index)
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
