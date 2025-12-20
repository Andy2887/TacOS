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

### To Do
1, Check test case test/schedule/priority/preempt.rs, figure out what schedule() should do exactly
2, Fix condvar test case