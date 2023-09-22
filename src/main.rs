mod machine;
mod ram;

use crate::machine::Machine;
use crate::ram::Ram;


fn main() {
    let ram = Ram::new(vec![
        // li	ra,1000
        0x93, 0x00, 0x80, 0x3e,
        // wfi
        0x73, 0x00, 0x50, 0x10,
        // nop
        0x13, 0x00, 0x00, 0x00,
    ]);
    let mut m = Machine::new(ram);
    m.tick();
}
