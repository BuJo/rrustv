
test.elf: test.c test.ld
	riscv64-unknown-linux-gnu-gcc -c test.c -march=rv64i -mabi=lp64 -O0 -g -std=c11 -fdebug-prefix-map=$(pwd)=.
	riscv64-unknown-linux-gnu-ld -static -Ttest.ld test.o -o test.elf
