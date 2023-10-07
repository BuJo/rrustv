use std::sync::RwLock;

use crate::device::Device;
use crate::plic::Fault;
use crate::plic::Fault::MemoryFault;

pub const DRAM_SIZE: usize = 1024 * 1024 * 128; // 128MiB

pub struct Ram {
    data: RwLock<Vec<u8>>,
}

impl Ram {
    pub fn new() -> Ram {
        let ram = vec![0; DRAM_SIZE];

        Self {
            data: RwLock::new(ram),
        }
    }

    pub fn size(&self) -> usize {
        return DRAM_SIZE;
    }

    pub fn write(&self, addr: usize, code: Vec<u8>) -> Option<()> {
        let mut shared = self.data.write().unwrap();

        shared.splice(addr..(addr + code.len()), code.iter().cloned());
        Some(())
    }
}

impl Device for Ram {
    fn write_word(&self, addr: usize, word: u32) -> Result<(), Fault> {
        let mut shared = self.data.write().unwrap();

        *(shared.get_mut(addr).ok_or(MemoryFault(addr))?) = (word & 0xFF) as u8;
        *(shared.get_mut(addr + 1).ok_or(MemoryFault(addr))?) = ((word >> 8) & 0xFF) as u8;
        *(shared.get_mut(addr + 2).ok_or(MemoryFault(addr))?) = ((word >> 16) & 0xFF) as u8;
        *(shared.get_mut(addr + 3).ok_or(MemoryFault(addr))?) = ((word >> 24) & 0xFF) as u8;

        Ok(())
    }

    fn write_byte(&self, addr: usize, val: u8) -> Result<(), Fault> {
        let mut shared = self.data.write().unwrap();

        *(shared.get_mut(addr).ok_or(MemoryFault(addr))?) = val;
        Ok(())
    }

    fn read_word(&self, addr: usize) -> Result<u32, Fault> {
        let data = self.data.read().unwrap();

        let ins: u32 = (*data.get(addr).ok_or(MemoryFault(addr))? as u32)
            + ((*data.get(addr + 1).ok_or(MemoryFault(addr))? as u32) << 8)
            + ((*data.get(addr + 2).ok_or(MemoryFault(addr))? as u32) << 16)
            + ((*data.get(addr + 3).ok_or(MemoryFault(addr))? as u32) << 24);
        Ok(ins)
    }

    fn read_byte(&self, addr: usize) -> Result<u8, Fault> {
        let data = self.data.read().unwrap();

        data.get(addr).copied().ok_or(MemoryFault(addr))
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
