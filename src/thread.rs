//! Kernel Threads

mod imp;
pub mod manager;
pub mod scheduler;
pub mod switch;
use crate::sync::Lock;
use crate::thread;

pub use self::imp::*;
pub use self::manager::Manager;
pub(self) use self::scheduler::{Schedule, Scheduler};
use crate::sbi::interrupt;
use crate::sync::Lazy;
use core::sync::atomic::Ordering;

use alloc::{collections::btree_map::BTreeMap, sync::Arc, vec::Vec};

/// Create a new thread
pub fn spawn<F>(name: &'static str, f: F) -> Arc<Thread>
where
    F: FnOnce() + Send + 'static,
{
    Builder::new(f).name(name).spawn()
}

/// Get the current running thread
pub fn current() -> Arc<Thread> {
    Manager::get().current.lock().clone()
}

/// Yield the control to another thread (if there's another one ready to run).
pub fn schedule() {
    Manager::get().schedule()
}

/// Gracefully shut down the current thread, and schedule another one.
pub fn exit() -> ! {
    {
        let current = Manager::get().current.lock();

        #[cfg(feature = "debug")]
        kprintln!("Exit: {:?}", *current);

        current.set_status(Status::Dying);
    }

    schedule();

    unreachable!("An exited thread shouldn't be scheduled again");
}

/// Mark the current thread as [`Blocked`](Status::Blocked) and
/// yield the control to another thread
pub fn block() {
    let current = current();
    current.set_status(Status::Blocked);

    #[cfg(feature = "debug")]
    kprintln!("[THREAD] Block {:?}", current);

    schedule();
}

/// Wake up a previously blocked thread, mark it as [`Ready`](Status::Ready),
/// and register it into the scheduler.
pub fn wake_up(thread: Arc<Thread>) {
    #[cfg(feature = "debug")]
    kprintln!(
        "[THREAD] Wake up {:?} with priority {}",
        thread,
        thread.effective_priority()
    );
    assert_eq!(thread.status(), Status::Blocked);
    thread.set_status(Status::Ready);

    Manager::get().scheduler.lock().register(thread.clone());

    // If the new waken up thread has higher priority than the current thread,
    // the current thread will yield

    #[cfg(feature = "debug")]
    kprintln!(
        "[THREAD] current thread priority: {}",
        thread::get_priority()
    );

    if thread.effective_priority() > thread::get_priority() {
        #[cfg(feature = "debug")]
        kprintln!("[THREAD] Schedule() is called!");
        schedule();
    } else {
        #[cfg(feature = "debug")]
        kprintln!("[THREAD] Schedule() is not called!");
    }
}

/// (Lab1) Sets the current thread's priority to a given value
pub fn set_priority(priority: u32) {
    let current_thread = current();

    current_thread.priority.store(priority, Ordering::Relaxed);

    #[cfg(feature = "debug")]
    kprintln!(
        "[DEBUG set_priority] thread_id: {}, new priority: {}",
        current_thread.id(),
        priority
    );

    let highest_priority = Manager::get().scheduler.lock().highest_priority();

    #[cfg(feature = "debug")]
    kprintln!(
        "[DEBUG set_priority] Highest priority in scheduler: {}",
        highest_priority
    );

    if priority < highest_priority {
        #[cfg(feature = "debug")]
        kprintln!("[DEBUG set_priority] schedule() called! ");
        schedule();
    } else {
        #[cfg(feature = "debug")]
        kprintln!("[DEBUG set_priority] Not calling schedule()!");
    }
}

/// (Lab1) Returns the current thread's effective priority.
pub fn get_priority() -> u32 {
    current().effective_priority()
}

pub static SLEEP_LIST: Lazy<Mutex<BTreeMap<i64, Vec<Arc<Thread>>>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));

/// (Lab1) Make the current thread sleep for the given ticks.
pub fn sleep(ticks: i64) {
    use crate::sbi::timer::timer_ticks;

    let old = interrupt::set(false);

    // let start = timer_ticks();

    // while timer_elapsed(start) < ticks {
    //     schedule();
    // }

    if ticks <= 0 {
        return;
    }

    // Calculate when the thread should wake up
    let wake_tick = timer_ticks() + ticks;
    let current_thread = current();

    // Push the new thread to SLEEP_LIST
    SLEEP_LIST
        .lock()
        .entry(wake_tick)
        .or_default()
        .push(current_thread);

    interrupt::set(old);

    // Block the current thread
    block();
}

pub fn donate_to(donor: Arc<Thread>, receiver: Arc<Thread>) {
    #[cfg(feature = "debug")]
    kprintln!(
        "[DEBUG donate_to] Donating - donor: {}, receiver: {}",
        donor.id(),
        receiver.id()
    );

    let mut donors = donor.donors();

    donors.insert(donor.id(), donor.clone());

    for (_, thread) in donors {
        receiver.add_donor(thread);
    }

    // Update the scheduler since receiver's effective priority changes
    Manager::get()
        .scheduler
        .lock()
        .change_priority(receiver.clone(), receiver.effective_priority());

    if let Some(lock) = receiver.waits_on() {
        let next_receiver = lock.holder().unwrap();

        if next_receiver.effective_priority() < receiver.effective_priority() {
            donate_to(donor, next_receiver);
        }
    }
}

pub fn remove_donation(donor: Arc<Thread>, receiver: Arc<Thread>) {
    #[cfg(feature = "debug")]
    kprintln!(
        "[DEBUG remove_donation] Removing donation - donor: {}, receiver: {}",
        donor.id(),
        receiver.id()
    );
    receiver.remove_donor(donor.clone());

    Manager::get()
        .scheduler
        .lock()
        .change_priority(receiver.clone(), receiver.effective_priority());

    if let Some(lock) = receiver.waits_on() {
        let next_receiver = lock.holder().unwrap();
        remove_donation(donor, next_receiver);
    }
}
