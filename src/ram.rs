use std::sync::RwLock;

pub const DRAM_SIZE: usize = 1024 * 1024 * 128; // 128MiB

pub struct Ram {
    ram: RwLock<Vec<u8>>,
}

impl Ram {
    pub fn new() -> Ram {
        let ram = vec![0; DRAM_SIZE];

        Self {
            ram: RwLock::new(ram),
        }
    }

    pub fn write(&self, addr: usize, code: Vec<u8>) {
        let mut shared = self.ram.write().unwrap();

        shared.splice(addr..(addr + code.len()), code.iter().cloned());
    }

    pub fn write_word(&self, addr: usize, word: u32) {
        let mut shared = self.ram.write().unwrap();

        shared[addr] = (word & 0xFF) as u8;
        shared[addr + 1] = ((word >> 8) & 0xFF) as u8;
        shared[addr + 2] = ((word >> 16) & 0xFF) as u8;
        shared[addr + 3] = ((word >> 24) & 0xFF) as u8;
    }

    pub fn write_byte(&self, addr: usize, val: u8) {
        let mut shared = self.ram.write().unwrap();

        shared[addr] = val
    }

    pub fn read_word(&self, addr: usize) -> Option<u32> {
        let ram = self.ram.read().unwrap();

        let ins: u32 = (*ram.get(addr)? as u32)
            + ((*ram.get(addr + 1)? as u32) << 8)
            + ((*ram.get(addr + 2)? as u32) << 16)
            + ((*ram.get(addr + 3)? as u32) << 24);
        Some(ins)
    }

    pub fn read_byte(&self, addr: usize) -> Option<u8> {
        let shared = self.ram.read().unwrap();

        //eprintln!("reading byte: {} at addr 0x{:04x}", self.ram[addr], addr);
        shared.get(addr).copied()
    }
}

#[cfg(test)]
mod tests {
    use crate::ram::Ram;

    #[test]
    fn init_read() {
        let ram = Ram::new();
        ram.write(0, vec![0x13, 0x81, 0x00, 0x7d]);
        let i = ram.read_word(0).expect("read");

        assert_eq!(i, 0x7d008113, "x1 mismatch");
    }

    #[test]
    fn write_read_cycle() {
        let ram = Ram::new();
        ram.write(0, vec![0x13, 0x81, 0x00, 0x7d]);
        ram.write_word(0, 0xdeadbeef);
        let i = ram.read_word(0).expect("read");

        assert_eq!(i, 0xdeadbeef, "dead beef");
    }
}
