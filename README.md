# Tacos

> Pintos reimplemented in Rust for riscv64.

This repo contains skeleton code for undergraduate Operating System course honor track at Peking University.

# Documentation

[Tacos Documentation](https://pku-tacos.pages.dev/appendix/debugging)

### Note
1, In Appendix 10 Debugging Print section, there is a typo.

Original: `Workspace/> cargo -F test-user,debug -- -append args-none`
Fixed: `Workspace/> cargo run -F test-user,debug -- -append args-none`

2, Some of the dependencies in the enviornment need to be depreciated.

### Fast commands
priority-condvar Test Case:
Print Debugging:
- `cargo run -F test-schedule,debug -- -append priority-preempt`
Debugger (in tool):
- `cargo gdb -c priority-condvar`

### Plan
Priority Donation Plan
Plan:
1, In sleep.rs,
the sleep lock will keep track of the thread currently holding the lock.

acquire(): Assume L holds a lock and H tries to acquire it, we add H's information to "donation" field in L.

If L is also waiting for another lock, we will recursively call donate() to pass the donation down.

We will also update the priority in the scheduler in our donate() function.

release(): We remove H's information from "donation" field from L.

2, In imp.rs
We add a new field called "donation". This is a BTreeMap storing all of the donating threads. The key is the **actual priority** of the thread, and the value is the reference of the thread.

Add a "wait_on" field, this stores the lock the thread is waiting on. 

We will also modify get_priority() to return effective priority


Example 1: multiple donation

1, We have L holding the lock.

2, We have H1, H2, H3 trying to acquire the lock.

In this case, H1, H2 and H3 will call donate() function and add themselves to the "donation" deque of L.

In the donate() function, L's priority in scheduler will be updated.

3, After L finishes using, donation field will be cleared, L's priority will be demoted to its ordinary priority. We remove "lock_owner" from the lock. We will call schedule() so that L yield to scheduler.

Example 2: chained donation

1, We have L holding lock A.

2, We have M holding lock B, and it tries to acquire lock A.

In this case, he will donate to A. 

3, We have H trying to acquire lock B.

In this case, he will call donate(). Donate will recursively being called, because it checks "waits_on" field of M and discoveres that M is also waiting for another field. Each donate() also updates the scheduler.

Note: the donate() function must pass all the donations of the current thread down to the next thread.

For example, assume M starts working, H donates to M, and M tries to acquire a lock that L has. In this case, M must contribute all of its donations down to L so that L has H's priority.


##### To Do

1, Update functions to use priority() instead of load
