pub const DRAM_SIZE: usize = 1024 * 1024 * 128; // 128MiB

pub struct Ram {
    pub ram: Vec<u8>,
}

impl Ram {
    pub fn new(code: Vec<u8>) -> Ram {
        let mut ram = vec![0; DRAM_SIZE];
        ram.splice(..code.len(), code.iter().cloned());

        Self { ram }
    }

    pub fn write_word(&mut self, addr: usize, word: u32) {
        self.ram[addr + 0] = ((word >> 0) & 0xFF) as u8;
        self.ram[addr + 1] = ((word >> 8) & 0xFF) as u8;
        self.ram[addr + 2] = ((word >> 16) & 0xFF) as u8;
        self.ram[addr + 3] = ((word >> 24) & 0xFF) as u8;
    }

    pub fn read_word(&mut self, addr: usize) -> u32 {
        let ins: u32 =
            0 +
                ((self.ram[addr + 0] as u32) << 0) +
                ((self.ram[addr + 1] as u32) << 8) +
                ((self.ram[addr + 2] as u32) << 16) +
                ((self.ram[addr + 3] as u32) << 24);
        ins
    }
}


#[cfg(test)]
mod tests {
    use crate::Machine;
    use crate::ram::Ram;

    #[test]
    fn init_read() {
        let mut ram = Ram::new(vec![0x13, 0x81, 0x00, 0x7d]);
        let i = ram.read_word(0);

        assert_eq!(i, 0x7d008113, "x1 mismatch");
    }
}