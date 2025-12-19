use alloc::collections::btree_map::BTreeMap;
use alloc::collections::vec_deque::VecDeque;
use alloc::sync::Arc;
use core::cell::{Cell, RefCell};

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
#[derive(Clone)]
pub struct Semaphore {
    value: Cell<usize>,
    waiters: RefCell<BTreeMap<u32, VecDeque<Arc<Thread>>>>,
}

unsafe impl Sync for Semaphore {}
unsafe impl Send for Semaphore {}

impl Semaphore {
    /// Creates a new semaphore of initial value n.
    pub const fn new(n: usize) -> Self {
        Semaphore {
            value: Cell::new(n),
            waiters: RefCell::new(BTreeMap::new()),
        }
    }

    /// P operation
    pub fn down(&self) {
        let old = sbi::interrupt::set(false);

        // Is semaphore available?
        while self.value() == 0 {
            let priority = thread::get_priority();
            self.waiters
                .borrow_mut()
                .entry(priority)
                .or_insert_with(VecDeque::new)
                .push_back(thread::current());
            // Block the current thread until it's awakened by an `up` operation
            thread::block();
        }
        self.value.set(self.value() - 1);

        sbi::interrupt::set(old);
    }

    /// V operation
    pub fn up(&self) {
        let old = sbi::interrupt::set(false);
        let count = self.value.replace(self.value() + 1);

        // Check if we need to wake up a sleeping waiter (highest priority first)
        let mut waiters = self.waiters.borrow_mut();
        if let Some(&priority) = waiters.keys().next_back() {
            assert_eq!(count, 0);

            let thread = waiters
                .get_mut(&priority)
                .and_then(|deque| deque.pop_front());

            // Remove the priority entry if the deque is now empty
            if waiters.get(&priority).map_or(false, |d| d.is_empty()) {
                waiters.remove(&priority);
            }

            if let Some(t) = thread {
                thread::wake_up(t);
            }
        }

        sbi::interrupt::set(old);
    }

    /// Get the current value of a semaphore
    pub fn value(&self) -> usize {
        self.value.get()
    }
}
