mod hart;
mod ram;
mod csr;
mod see;

use std::fs;
use crate::hart::Hart;
use crate::ram::Ram;


fn main() {
    let text = fs::read("target/target.text").expect("no .text");
    let data = fs::read("target/target.data").expect("no .data");

    let mut ram = Ram::new(text);
    ram.write(0x1000, data);

    let mut m = Hart::new(ram);
    for _ in 0..100 {
        m.tick()
    }
}
