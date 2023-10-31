#!/bin/bash

set -e -o pipefail

DIR=`dirname $(readlink -f $0)`

tst=$1
work=$DIR/riscof_work

cargo build
../target/debug/archtest $work/$tst/dut/my.elf $work/$tst/dut/DUT-rriscv.signature 2>$work/$tst/dut/DUT.disass || true
./clean-disass.sh $work/$tst/dut/DUT.disass $work/$tst/ref/ref.disass

echo meld $work/$tst/dut/DUT.disass $work/$tst/ref/ref.disass
echo meld $work/$tst/dut/DUT-rriscv.signature $work/$tst/ref/Reference-sail_c_simulator.signature
