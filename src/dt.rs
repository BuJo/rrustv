use std::fs;

pub fn load(x: &str) -> Vec<u8> {
    fs::read(format!("data/{x}.dtb")).expect("no device tree data")
}
