use alloc::sync::Arc;
use core::cell::RefCell;

#[cfg(feature = "debug")]
use crate::kprintln;
use crate::sync::{Lock, Semaphore};
use crate::thread::{self, current, donate_to, Thread};
use core::sync::atomic::{AtomicUsize, Ordering};

/// Sleep lock. Uses [`Semaphore`] under the hood.
pub struct Sleep {
    pub id: usize,
    inner: Semaphore,
    holder: RefCell<Option<Arc<Thread>>>,
}

impl Default for Sleep {
    fn default() -> Self {
        Self {
            id: generate_id(),
            inner: Semaphore::new(1),
            holder: Default::default(),
        }
    }
}

// A global counter that starts at 0
static NEXT_LOCK_ID: AtomicUsize = AtomicUsize::new(0);

fn generate_id() -> usize {
    // fetch_add returns the previous value and increments it by 1 atomically
    NEXT_LOCK_ID.fetch_add(1, Ordering::SeqCst)
}

impl Lock for Sleep {
    fn acquire(&self) {
        #[cfg(feature = "debug")]
        kprintln!(
            "[DEBUG acquire] Thread {} trying to acquire sleep lock",
            thread::current().id()
        );

        let current = current();
        if let Some(holder) = self.holder() {
            if holder.effective_priority() < current.effective_priority() {
                donate_to(current.clone(), holder.clone(), self.id);
                current.set_waits_on(Some(holder), Some(self.id));
            }
        }

        self.inner.down();
        self.holder.borrow_mut().replace(thread::current());
        current.set_waits_on(None, None);

        #[cfg(feature = "debug")]
        kprintln!(
            "[DEBUG acquire] Thread {} acquired sleep lock",
            thread::current().id()
        );
    }

    fn release(&self) {
        #[cfg(feature = "debug")]
        kprintln!(
            "[DEBUG release] Thread {} releasing sleep lock",
            thread::current().id()
        );

        assert!(Arc::ptr_eq(
            self.holder.borrow().as_ref().unwrap(),
            &thread::current()
        ));

        let holder = self.holder.borrow_mut().take().unwrap();
        holder.remove_donors(self.id);
        self.inner.up();
    }

    fn holder(&self) -> Option<Arc<Thread>> {
        self.holder.borrow().clone()
    }
}

unsafe impl Sync for Sleep {}
