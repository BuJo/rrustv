use std::sync::RwLock;

pub struct Rom {
    data: RwLock<Vec<u8>>,
}

impl Rom {
    pub fn new(data: Vec<u8>) -> Rom {
        Self {
            data: RwLock::new(data),
        }
    }

    pub fn read_word(&self, addr: usize) -> Option<u32> {
        let data = self.data.read().unwrap();

        let ins: u32 = (*data.get(addr)? as u32)
            + ((*data.get(addr + 1)? as u32) << 8)
            + ((*data.get(addr + 2)? as u32) << 16)
            + ((*data.get(addr + 3)? as u32) << 24);
        Some(ins)
    }

    pub fn read_byte(&self, addr: usize) -> Option<u8> {
        let data = self.data.read().unwrap();

        //eprintln!("reading byte: {} at addr 0x{:04x}", self.ram[addr], addr);
        data.get(addr).copied()
    }
}

#[cfg(test)]
mod tests {
    use crate::rom::Rom;

    #[test]
    fn init_read() {
        let ram = Rom::new(vec![0x13, 0x81, 0x00, 0x7d]);
        let i = ram.read_word(0).expect("read");

        assert_eq!(i, 0x7d008113, "x1 mismatch");
    }
}
