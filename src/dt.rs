use std::fs;

pub fn load() -> Vec<u8> {
    fs::read("data/rriscv.dtb").expect("no device tree data")
}
