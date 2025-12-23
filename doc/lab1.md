# Lab 1: Scheduling

---

## Information

Name:

Email:

> Please cite any forms of information source that you have consulted during finishing your assignment, except the TacOS documentation, course slides, and course staff.

> With any comments that may help TAs to evaluate your work better, please leave them here

## Alarm Clock

### Data Structures

> A1: Copy here the **declaration** of every new or modified struct, enum type, and global variable. State the purpose of each within 30 words.

```rust
pub static SLEEP_LIST: Lazy<Mutex<BTreeMap<i64, Vec<Arc<Thread>>>>> =
    Lazy::new(|| Mutex::new(BTreeMap::new()));
```
This SLEEP_LIST is used to keep track of all the sleeping threads. A Mutex is added to ensure syncronization.

### Algorithms

> A2: Briefly describe what happens in `sleep()` and the timer interrupt handler.

1. in `sleep()`, we will calculate the wake up tick for the current thread, and add the wake up tick and curren thread as pair into SLEEP_LIST

2, in the timer interrupt handler, we will call check_sleeping_threads(). This function will compare the current time and the wake_up time in SLEEP_LIST. If it is the wake up time, we will wake up all the threads associated with this time.

> A3: What are your efforts to minimize the amount of time spent in the timer interrupt handler?

I use BTreeMap to store the sleeping threads based on priority. This gives me O(1) time complexity when finding the threads with highest priority, compared to O(n) using a normal deque.

### Synchronization

> A4: How are race conditions avoided when `sleep()` is being called concurrently?

I use a lock on SLEEP_LIST.

> A5: How are race conditions avoided when a timer interrupt occurs during a call to `sleep()`?

I turn off timer interrupt.

## Priority Scheduling

### Data Structures

> B1: Copy here the **declaration** of every new or modified struct, enum type, and global variable. State the purpose of each within 30 words.

> B2: Explain the data structure that tracks priority donation. Clarify your answer with any forms of diagram (e.g., the ASCII art).

In thread, I use a BTreeMap to store the donating threads. I also have a "waits_on" field that points to the lock the thread is waiting on.

In the lock, I added a field called "owner" to keep track of which thread has the lock.

### Algorithms

> B3: How do you ensure that the highest priority thread waiting for a lock, semaphore, or condition variable wakes up first?

I use a BTreeMap. The key is priority, and the value is a deque storing thread atomic references. Whenever we need to wake up a thread, we always look for the one with the highest priority in BTreeMap.

> B4: Describe the sequence of events when a thread tries to acquire a lock. How is nested donation handled?

Assume we have H -> M -> L.

1, H tries to acquire lock. It checks owner, but it has an owner M.

2, H calls donate(). This function donate to M. It also checks if M waits on any lock. If there is, donate() is called recursively. H and M's donate will be passed on to L.

> B5: Describe the sequence of events when a lock, which a higher-priority thread is waiting for, is released.

Assume we have H -> L.

1, L clears its donation BTreeMap, remove itself from "owner" of lock, and calls schedule() to yield.

2, H will use the lock, and add itself as the owner of the lock.

### Synchronization

> B6: Describe a potential race in `thread::set_priority()` and explain how your implementation avoids it. Can you use a lock to avoid this race?

If the implementation used the WRONG order (check scheduler first, then store priority), it will cause a potential race if a thread wakes up in the middle of set_priority().

Example:

Thread A (current, priority 30) wants to lower to priority 10

Thread B (blocked, priority 20) is being woken up concurrently

Scheduler has highest priority of 5.

Flow:

1, Thread A checks highest priority: 5

2, Thread B wakes up, register thread B to scheduler

3, Thread B has lower priority than current thread, so the current keeps running

4, Thread A changes its priority to 10

5, Thread A compares 10 to 5, and decides to keep running.

My implementation is in the right order.

If the thread wakes up after I store priority

1, Thread A stores priority 10

2, Thread B wakes up, register thread B to scheduler

3, Thread B has higher priority than current thread (20 > 10), so Thread B starts running

4, Thread A runs after thread B

## Rationale

> C1: Have you considered other design possibilities? You can talk about anything in your solution that you once thought about doing them another way. And for what reasons that you made your choice?

I thought about using deque instead of BTreeMap when storing "donating threads". This is because at first I believed that storing the "effective priority" as key does not make sense because you need to keep changing it in a "chained" donation situation. However, I changed my mind and use BTreeMap to store **actual priority** of all the donating threads. This avoids constant changing the "key" and ensures speed. 