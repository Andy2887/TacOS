use alloc::sync::Arc;
use core::cell::RefCell;

#[cfg(feature = "debug")]
use crate::kprintln;
use crate::sync::{Lock, Semaphore};
use crate::thread::{self, current, donate_to, Thread};

/// Sleep lock. Uses [`Semaphore`] under the hood.
pub struct Sleep {
    id: usize,
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

fn generate_id() -> usize {
    let val = 0;
    &val as *const i32 as usize
}

impl Lock for Sleep {
    fn acquire(&self) {
        #[cfg(feature = "debug")]
        kprintln!(
            "[DEBUG acquire] Thread {} acquiring sleep lock",
            thread::current().id()
        );

        let current = current();
        if let Some(holder) = self.holder() {
            if holder.effective_priority() < current.effective_priority() {
                donate_to(current.clone(), holder, self.id);
            }
        }

        self.inner.down();
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

        let holder = self.holder.borrow_mut().take().unwrap();
        holder.remove_donors(self.id);
        self.inner.up();
    }

    fn holder(&self) -> Option<Arc<Thread>> {
        self.holder.borrow().clone()
    }
}

unsafe impl Sync for Sleep {}
