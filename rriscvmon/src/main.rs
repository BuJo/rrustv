#![no_std]
#![no_main]

use core::arch::global_asm;

// Minimal machine-mode entry stub.
// Must run before any Rust code:
//   - point sp at our stack
//   - zero BSS
//   - call main, hang forever if it returns
global_asm!(
    ".section .text.entry",
    ".global _start",
    "_start:",
    // Set up the stack pointer
    "   la   sp, _stack_top",
    // Zero .bss
    "   la   t0, _bss_start",
    "   la   t1, _bss_end",
    "1: bgeu t0, t1, 2f",
    "   sd   zero, 0(t0)",       // use sw instead for rv32
    "   addi t0, t0, 8",         // use 4 for rv32
    "   j    1b",
    "2:",
    // Jump into Rust
    "   call main",
    // main must never return; spin if it does
    "3: j    3b",
);

#[unsafe(no_mangle)]
pub extern "C" fn main() -> ! {
    // Your WozMon logic will live here.
    loop {}
}

// Required by no_std — panic just halts.
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}
