#!/bin/bash

sync

dut="$1"
ref="$2"

sed -i 's/\(lui.*,0x\)fff\(.....$\)/\1\2/' $dut
sed -i 's/\(lui.*,0x\)fff\(.....$\)/\1\2/' $dut
sed -i 's/add.*zero,zero,0 # 0/nop/' $dut
sed -i 's/add\(.*\),zero,\(-*[0-9]*\) # .*/li\1,\2/' $dut
sed -i 's/jal\(.*\)zero,\([0-9]*\)/j\1\2/' $dut
sed -i 's/jr\(.*\),0x0$/jr\1/' $dut
sed -i 's/add\(.*\),0 #.*$/mv\1/' $dut
sed -i 's/csrrs\(.*\),zero$/csrr\1/' $dut
sed -i 's/csrrw\tzero,\(.*\)$/csrw\t\1/' $dut
sed -i 's/xor\(.*\),-1 /not\1 /' $dut
sed -i '/Opcode for ins/d' $dut

sed -i 's/ <.*$//' $ref
sed -i '/^80......$/d' $ref
sed -i '/jbuch/d' $ref
sed -i '/Disassem/d' $ref
sed -i '/littleriscv/d' $ref
sed -i '/^$/d' $ref
sed -i '/^[0-9a-f]*$/d' $ref

sed -i '/nop/d' $ref $dut
sed -i 's/ #.*$//' $ref $dut

