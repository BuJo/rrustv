use std::sync::RwLock;

use crate::device::Device;
use crate::plic::Fault;
use crate::plic::Fault::MemoryFault;

pub struct Rom {
    data: RwLock<Vec<u8>>,
}

impl Rom {
    pub fn new(data: Vec<u8>) -> Rom {
        Self {
            data: RwLock::new(data),
        }
    }

    pub fn len(&self) -> usize {
        let data = self.data.read().unwrap();

        data.len()
    }

    pub fn is_empty(&self) -> bool {
        let data = self.data.read().unwrap();

        data.is_empty()
    }
}

impl Device for Rom {
    fn write_double(&self, addr: usize, _val: u64) -> Result<(), Fault> {
        Err(MemoryFault(addr))
    }

    fn write_word(&self, addr: usize, _val: u32) -> Result<(), Fault> {
        Err(MemoryFault(addr))
    }

    fn write_half(&self, addr: usize, _val: u16) -> Result<(), Fault> {
        Err(MemoryFault(addr))
    }

    fn write_byte(&self, addr: usize, _val: u8) -> Result<(), Fault> {
        Err(MemoryFault(addr))
    }

    fn read_double(&self, addr: usize) -> Result<u64, Fault> {
        let data = self.data.read().unwrap();

        let val = (*data.get(addr).ok_or(MemoryFault(addr))? as u64)
            + ((*data.get(addr + 1).ok_or(MemoryFault(addr))? as u64) << 8)
            + ((*data.get(addr + 2).ok_or(MemoryFault(addr))? as u64) << 16)
            + ((*data.get(addr + 3).ok_or(MemoryFault(addr))? as u64) << 24)
            + ((*data.get(addr + 3).ok_or(MemoryFault(addr))? as u64) << 32)
            + ((*data.get(addr + 3).ok_or(MemoryFault(addr))? as u64) << 40)
            + ((*data.get(addr + 3).ok_or(MemoryFault(addr))? as u64) << 48)
            + ((*data.get(addr + 3).ok_or(MemoryFault(addr))? as u64) << 56);
        Ok(val)
    }

    fn read_word(&self, addr: usize) -> Result<u32, Fault> {
        let data = self.data.read().unwrap();

        let val = (*data.get(addr).ok_or(MemoryFault(addr))? as u32)
            + ((*data.get(addr + 1).ok_or(MemoryFault(addr))? as u32) << 8)
            + ((*data.get(addr + 2).ok_or(MemoryFault(addr))? as u32) << 16)
            + ((*data.get(addr + 3).ok_or(MemoryFault(addr))? as u32) << 24);
        Ok(val)
    }

    fn read_half(&self, addr: usize) -> Result<u16, Fault> {
        let data = self.data.read().unwrap();

        let val = (*data.get(addr).ok_or(MemoryFault(addr))? as u16)
            + ((*data.get(addr + 1).ok_or(MemoryFault(addr))? as u16) << 8);
        Ok(val)
    }

    fn read_byte(&self, addr: usize) -> Result<u8, Fault> {
        let data = self.data.read().unwrap();

        data.get(addr).copied().ok_or(MemoryFault(addr))
    }
}

#[cfg(test)]
mod tests {
    use crate::device::Device;
    use crate::rom::Rom;

    #[test]
    fn init_read() {
        let ram = Rom::new(vec![0x13, 0x81, 0x00, 0x7d]);
        let i = ram.read_word(0).expect("read");

        assert_eq!(i, 0x7d008113, "x1 mismatch");
    }
}
