use std::fs;

fn load() -> Vec<u8> {
    fs::read("data/rriscv.dtb").expect("no .data")
}
