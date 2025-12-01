# Lab 0: Appetizer

---

## Information

Name:

Email:

> Please cite any forms of information source that you have consulted during finishing your assignment, except the TacOS documentation, course slides, and course staff.

> With any comments that may help TAs to evaluate your work better, please leave them here

## Booting Tacos

> A1: Put the screenshot of Tacos running example here.

## Debugging

### First instruction

> B1: What is the first instruction that gets executed?

=> 0x1000:      auipc   t0,0x0

> B2: At which physical address is this instruction located?

0x1000

### From ZSBL to SBI

> B3: Which address will the ZSBL jump to?

0x80000000

### SBI, kernel and argument passing

> B4: What's the value of the argument `hard_id` and `dtb`?

`hard_id` == 0
`dtb` == 2183135232

> B5: What's the value of `Domain0 Next Address`, `Domain0 Next Arg1`, `Domain0 Next Mode` and `Boot HART ID` in OpenSBI's output?

`Domain0 Next Address` == 0x0000000080200000
`Domain0 Next Arg1` == 0x0000000082200000
`Domain0 Next Mode` == S-mode
`Boot HART ID` == 0

> B6: What's the relationship between the four output values and the two arguments?

The OpenSBI output values are passed as arguments to the kernel's `main` function:

- `Domain0 Next Address` (0x80200000) is the entry point where OpenSBI jumps to start the kernel
- `Domain0 Next Arg1` (0x82200000) is passed as the `dtb` argument (2183135232 in decimal = 0x82200000 in hex)
- `Boot HART ID` (0) is passed as the `hart_id` argument

OpenSBI follows the RISC-V boot protocol where it passes the hart ID in register `a0` and the device tree blob (DTB) pointer in register `a1` before jumping to the kernel at the "Next Address". The kernel's `main` function receives these values as its two parameters. `Domain0 Next Mode` (S-mode) indicates the privilege level the kernel runs in (Supervisor mode).

### SBI interfaces

> B7: Inside `console_putchar`, Tacos uses `ecall` instruction to transfer control to SBI. What's the value of register `a6` and `a7` when executing that `ecall`?

`a6` == 0
`a7` == 1

## Kernel Monitor

> C1: Put the screenshot of your kernel monitor running example here. (It should show how your kernel shell respond to `whoami`, `exit`, and `other input`.)

Skipped

> C2: Explain how you read and write to the console for the kernel monitor.

**Writing to Console:**
The kernel monitor uses the `kprint!` and `kprintln!` macros to write output to the console. These macros use the `stdout()` function from `src/sbi/console.rs`, which locks the standard output and calls `console_putchar()`. The `console_putchar()` function makes an SBI ecall (with `a7=0x01` for CONSOLE_PUTCHAR) to request the SBI firmware to output each character to the console.

**Reading from Console:**
For reading user input, the kernel monitor calls `console_getchar()` from `src/sbi.rs` in a loop. This function makes an SBI ecall (with `a7=0x02` for CONSOLE_GETCHAR) to read a single character from the console input buffer. The returned character (as a `usize`) is converted to a `u8` byte and stored in a buffer. The reading loop continues until it encounters a newline character (`\n` or `\r`), at which point the buffered input is converted to a string and processed as a command.

### Extra Note

`ZSBL`: the very first piece of code executed by the processor immediately after power-on

`SBI`: Manages the hardware. OS needs to "ask" SBI to do stuff.
