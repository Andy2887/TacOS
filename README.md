# Tacos

> Pintos reimplemented in Rust for riscv64.

This repo contains skeleton code for undergraduate Operating System course honor track at Peking University.

# Documentation

[Tacos Documentation](https://pku-tacos.pages.dev/appendix/debugging)

### Note
1, In Appendix 10 Debugging Print section, there is a typo.

Original: `Workspace/> cargo -F test-user,debug -- -append args-none`
Fixed: `Workspace/> cargo run -F test-user,debug -- -append args-none`

2, Some of the dependencies in the enviornment need to be depreciated for the project to compile.

### Fast commands
donation-sema Test Case:
Print Debugging:
- `cargo run -F test-schedule,debug -- -append donation-sema`

##### To Do
1, fix donation-sema



