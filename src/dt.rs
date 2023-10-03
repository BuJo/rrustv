use std::fs;

pub(crate) fn load() -> Vec<u8> {
    fs::read("data/rriscv.dtb").expect("no device tree data")
}
