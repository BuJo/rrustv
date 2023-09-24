use std::sync::Mutex;

pub const DRAM_SIZE: usize = 1024 * 1024 * 128; // 128MiB

pub struct Ram {
    pub ram: Mutex<Vec<u8>>,
}

impl Ram {
    pub fn new(code: Vec<u8>) -> Ram {
        let mut ram = vec![0; DRAM_SIZE];
        ram.splice(..code.len(), code.iter().cloned());

        Self { ram: Mutex::new(ram) }
    }

    pub fn write(&self, addr: usize, code: Vec<u8>) {
        let mut shared = self.ram.lock().unwrap();

        shared.splice(addr..(addr + code.len()), code.iter().cloned());
    }

    pub fn write_word(&self, addr: usize, word: u32) {
        let mut shared = self.ram.lock().unwrap();

        shared[addr + 0] = ((word >> 0) & 0xFF) as u8;
        shared[addr + 1] = ((word >> 8) & 0xFF) as u8;
        shared[addr + 2] = ((word >> 16) & 0xFF) as u8;
        shared[addr + 3] = ((word >> 24) & 0xFF) as u8;
    }

    pub fn write_byte(&self, addr: usize, val: u8) {
        let mut shared = self.ram.lock().unwrap();

        shared[addr] = val
    }

    pub fn read_word(&self, addr: usize) -> u32 {
        let shared = self.ram.lock().unwrap();

        let ins: u32 =
            0 +
                ((shared[addr + 0] as u32) << 0) +
                ((shared[addr + 1] as u32) << 8) +
                ((shared[addr + 2] as u32) << 16) +
                ((shared[addr + 3] as u32) << 24);
        ins
    }

    pub fn read_byte(&self, addr: usize) -> u8 {
        let shared = self.ram.lock().unwrap();

        //eprintln!("reading byte: {} at addr 0x{:04x}", self.ram[addr], addr);
        shared[addr]
    }
}


#[cfg(test)]
mod tests {
    use crate::ram::Ram;

    #[test]
    fn init_read() {
        let mut ram = Ram::new(vec![0x13, 0x81, 0x00, 0x7d]);
        let i = ram.read_word(0);

        assert_eq!(i, 0x7d008113, "x1 mismatch");
    }

    #[test]
    fn write_read_cycle() {
        let mut ram = Ram::new(vec![0x13, 0x81, 0x00, 0x7d]);
        ram.write_word(0, 0xdeadbeef);
        let i = ram.read_word(0);

        assert_eq!(i, 0xdeadbeef, "dead beef");
    }
}
