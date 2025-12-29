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
- `cargo run -F test-schedule,debug -- -append priority-condvar`
Debugger (in tool):
- `cargo gdb -c priority-condvar`

##### To Do
1, fix priority-condvar

[603 ms] we are going to notify one thread
[619 ms] [THREAD] Wake up child(6)[Blocked] with priority 30
[620 ms] [REGISTER] tid: 6, priority: 30
[620 ms] [REGISTER] Other threads in scheduler (sorted by priority):
[620 ms] [REGISTER]   tid: 2, priority: 0
[621 ms] [THREAD] current thread priority: 0
[621 ms] [THREAD] Schedule() is called!
[622 ms] [REGISTER] tid: 0, priority: 0
[622 ms] [REGISTER] Other threads in scheduler (sorted by priority):
[623 ms] [REGISTER]   tid: 6, priority: 30
[623 ms] [REGISTER]   tid: 2, priority: 0
[623 ms] [SCHEDULE] Threads in scheduler (sorted by priority):
[624 ms] [SCHEDULE]   tid: 6, priority: 30
[624 ms] [SCHEDULE]   tid: 2, priority: 0
[624 ms] [SCHEDULE]   tid: 0, priority: 0
[624 ms] [SCHEDULE] Chosen thread - tid: 6, priority: 30
[625 ms] [THREAD] switch from test(0)[Ready]
[625 ms] [THREAD] switch to child(6)[Running]
[626 ms] [DEBUG donate_to] Donating - donor: 6, receiver: 0
[647 ms] [CHANGE_PRIORITY] BEFORE - tid: 0, priority: 0
[651 ms] [CHANGE_PRIORITY] AFTER - tid: 0, priority: 30
[652 ms] [TRAP] enter trap handler
[652 ms] [TRAP] Interrupt(SupervisorTimer), tval=0x0, sepc=0xffffffc080213218
[652 ms] [DEBUG] current_tick: 3
[653 ms] [DEBUG] sleep_list:
[653 ms] [DEBUG] sleep_list ends
[653 ms] [REGISTER] tid: 6, priority: 30
[653 ms] [REGISTER] Other threads in scheduler (sorted by priority):
[654 ms] [REGISTER]   tid: 0, priority: 30
[654 ms] [REGISTER]   tid: 2, priority: 0
[654 ms] [SCHEDULE] Threads in scheduler (sorted by priority):
[655 ms] [SCHEDULE]   tid: 0, priority: 30
[655 ms] [SCHEDULE]   tid: 6, priority: 30
[655 ms] [SCHEDULE]   tid: 2, priority: 0
[655 ms] [SCHEDULE] Chosen thread - tid: 0, priority: 30
[656 ms] [THREAD] switch from child(6)[Ready]
[656 ms] [THREAD] switch to test(0)[Running]

So in priority-condvar test case, the mutex is implemented using the sleep lock. I have modified sleep lock so that it will donate to the lock owner whenever the current thread has higher priority than the lock owner. 

This causes an error in the test case. What priority-condvar is doing is it creates a bunch of children threads and put them to sleep using a mutex and a condvar. Then, it wake them up one by one and finally the main thread exists. When the children thread wakes up and calls lock.lock() (condvar.rs:27), it donates to the owner of the lock - the main thread, blocking itself and yielding to main thread. Therefore, the child thread never finishes the job. 

So how could I solve this issue?

Note: the test case is at test/schedule/priority/condvar.rs, the sleep lock implementation is at src/sync/sleep.rs. You could also check src/thread.rs for relevant methods.

2, fix donation test cases
