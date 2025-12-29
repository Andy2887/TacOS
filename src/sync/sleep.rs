use alloc::sync::Arc;
use core::cell::RefCell;

#[cfg(feature = "debug")]
use crate::kprintln;
use crate::sync::{Lock, Semaphore};
use crate::thread::{self, current, donate_to, remove_donation, Thread};

/// Sleep lock. Uses [`Semaphore`] under the hood.
pub struct Sleep {
    inner: Semaphore,
    holder: RefCell<Option<Arc<Thread>>>,
}

impl Default for Sleep {
    fn default() -> Self {
        Self {
            inner: Semaphore::new(1),
            holder: Default::default(),
        }
    }
}

impl Lock for Sleep {
    fn acquire(&self) {
        #[cfg(feature = "debug")]
        kprintln!(
            "[DEBUG acquire] Thread {} acquiring sleep lock",
            thread::current().id()
        );

        let holder_option = self.holder();

        let mut donated = false;

        if let Some(ref holder) = holder_option {
            let current = current();
            if holder.effective_priority() < current.effective_priority() {
                donate_to(current, holder.clone());
                donated = true;
            }
        }

        self.inner.down();
        if donated {
            if let Some(holder) = holder_option {
                remove_donation(current(), holder);
            }
        }

        self.holder.borrow_mut().replace(thread::current());
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

        self.holder.borrow_mut().take().unwrap();
        self.inner.up();
    }

    fn holder(&self) -> Option<Arc<Thread>> {
        self.holder.borrow().clone()
    }
}

unsafe impl Sync for Sleep {}
