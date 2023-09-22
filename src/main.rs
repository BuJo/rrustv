mod machine;
mod ram;

use crate::machine::Machine;
use crate::ram::Ram;


fn main() {
    let mut ram = Ram::new(vec![
        // li	ra,1000
        0x93, 0x00, 0x80, 0x3e,
        // addi	sp,ra,2000
        0x13, 0x81, 0x00, 0x7d,
        // addi	gp,sp,-1000
        0x93, 0x01, 0x81, 0xc1,
        // addi	tp,gp,-2000
        0x13, 0x82, 0x01, 0x83,
        // addi	t0,tp,1000
        0x93, 0x02, 0x82, 0x3e,
        // li	t1,64
        0x13, 0x03, 0x00, 0x04,
        // addi	t1,t1,4
        0x13, 0x03, 0x43, 0x00,
    ]);
    ram.write_word(0x40, 0xdeadbeef);

    let mut m = Machine::new(ram);
    m.tick();
    m.tick();
    m.tick();
    m.tick();
    m.tick();
    m.tick();
    m.tick();
}
