#
# Risc-V Assembler program to print "Hello World!"
# to stdout.
#
# a0-a2 - parameters to sbi
# a7 - sbi system call number
#

.global _start      # Provide program starting address to linker

# Setup the Loop to print each byte from the .data segment
# and print each character with SBI legacy commands.

_start:
  la t0, helloworld    # i = beginning of hello world

_loop:
  lb    a0, 0(t0)       # load byte
  beq   a0, x0, _out    # if on 0 byte
  addi  a7, x0, 0x01    # sbi print char
  ecall                 # Call SEE
  addi  t0, t0, 1       # i++
  beq   x0, x0, _loop   # jump back in loop

_out:
  ebreak

.section .rodata
helloworld:      .ascii "Hello World!\n"

