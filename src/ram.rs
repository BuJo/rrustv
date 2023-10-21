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
        DRAM_SIZE
    }

    pub fn write(&self, addr: usize, code: Vec<u8>) -> Option<()> {
        let mut shared = self.data.write().unwrap();

        shared.splice(addr..(addr + code.len()), code.iter().cloned());
        Some(())
    }
}

impl Default for Ram {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for Ram {
    fn write_double(&self, addr: usize, val: u64) -> Result<(), Fault> {
        let mut shared = self.data.write().unwrap();

        *(shared.get_mut(addr).ok_or(MemoryFault(addr))?) = (val & 0xFF) as u8;
        *(shared.get_mut(addr + 1).ok_or(MemoryFault(addr))?) = ((val >> 8) & 0xFF) as u8;
        *(shared.get_mut(addr + 2).ok_or(MemoryFault(addr))?) = ((val >> 16) & 0xFF) as u8;
        *(shared.get_mut(addr + 3).ok_or(MemoryFault(addr))?) = ((val >> 24) & 0xFF) as u8;
        *(shared.get_mut(addr + 4).ok_or(MemoryFault(addr))?) = ((val >> 32) & 0xFF) as u8;
        *(shared.get_mut(addr + 5).ok_or(MemoryFault(addr))?) = ((val >> 40) & 0xFF) as u8;
        *(shared.get_mut(addr + 6).ok_or(MemoryFault(addr))?) = ((val >> 48) & 0xFF) as u8;
        *(shared.get_mut(addr + 7).ok_or(MemoryFault(addr))?) = ((val >> 56) & 0xFF) as u8;

        Ok(())
    }
    fn write_word(&self, addr: usize, val: u32) -> Result<(), Fault> {
        let mut shared = self.data.write().unwrap();

        *(shared.get_mut(addr).ok_or(MemoryFault(addr))?) = (val & 0xFF) as u8;
        *(shared.get_mut(addr + 1).ok_or(MemoryFault(addr))?) = ((val >> 8) & 0xFF) as u8;
        *(shared.get_mut(addr + 2).ok_or(MemoryFault(addr))?) = ((val >> 16) & 0xFF) as u8;
        *(shared.get_mut(addr + 3).ok_or(MemoryFault(addr))?) = ((val >> 24) & 0xFF) as u8;

        Ok(())
    }

    fn write_half(&self, addr: usize, val: u16) -> Result<(), Fault> {
        let mut shared = self.data.write().unwrap();

        *(shared.get_mut(addr).ok_or(MemoryFault(addr))?) = (val & 0xFF) as u8;
        *(shared.get_mut(addr + 1).ok_or(MemoryFault(addr))?) = ((val >> 8) & 0xFF) as u8;

        Ok(())
    }

    fn write_byte(&self, addr: usize, val: u8) -> Result<(), Fault> {
        let mut shared = self.data.write().unwrap();

        *(shared.get_mut(addr).ok_or(MemoryFault(addr))?) = val;
        Ok(())
    }

    fn read_double(&self, addr: usize) -> Result<u64, Fault> {
        let data = self.data.read().unwrap();

        let val: u64 = (*data.get(addr).ok_or(MemoryFault(addr))? as u64)
            + ((*data.get(addr + 1).ok_or(MemoryFault(addr))? as u64) << 8)
            + ((*data.get(addr + 2).ok_or(MemoryFault(addr))? as u64) << 16)
            + ((*data.get(addr + 3).ok_or(MemoryFault(addr))? as u64) << 24)
            + ((*data.get(addr + 4).ok_or(MemoryFault(addr))? as u64) << 32)
            + ((*data.get(addr + 5).ok_or(MemoryFault(addr))? as u64) << 40)
            + ((*data.get(addr + 6).ok_or(MemoryFault(addr))? as u64) << 48)
            + ((*data.get(addr + 7).ok_or(MemoryFault(addr))? as u64) << 56);
        Ok(val)
    }
    fn read_word(&self, addr: usize) -> Result<u32, Fault> {
        let data = self.data.read().unwrap();

        let val: u32 = (*data.get(addr).ok_or(MemoryFault(addr))? as u32)
            + ((*data.get(addr + 1).ok_or(MemoryFault(addr))? as u32) << 8)
            + ((*data.get(addr + 2).ok_or(MemoryFault(addr))? as u32) << 16)
            + ((*data.get(addr + 3).ok_or(MemoryFault(addr))? as u32) << 24);
        Ok(val)
    }

    fn read_half(&self, addr: usize) -> Result<u16, Fault> {
        let data = self.data.read().unwrap();

        let hw = (*data.get(addr).ok_or(MemoryFault(addr))? as u16)
            + ((*data.get(addr + 1).ok_or(MemoryFault(addr))? as u16) << 8);
        Ok(hw)
    }

    fn read_byte(&self, addr: usize) -> Result<u8, Fault> {
        let data = self.data.read().unwrap();

        data.get(addr).copied().ok_or(MemoryFault(addr))
    }
}

#[cfg(test)]
mod tests {
    use crate::device::Device;
    use crate::ram::Ram;

    #[test]
    fn init_read() {
        let ram = Ram::new();
        ram.write(0, vec![0x13, 0x81, 0x00, 0x7d]);
        let i = ram.read_word(0).expect("read");

        assert_eq!(i, 0x7d008113, "x1 mismatch");
    }

    #[test]
    fn write_read_cycle_u16() {
        let ram = Ram::new();
        ram.write_word(0, 0xdead).expect("written");
        let i = ram.read_half(0).expect("read");

        assert_eq!(i, 0xdead, "dead beef");
    }

    #[test]
    fn write_read_cycle_u32() {
        let ram = Ram::new();
        ram.write_word(0, 0xdeadbeef).expect("written");
        let i = ram.read_word(0).expect("read");

        assert_eq!(i, 0xdeadbeef, "dead beef");
    }

    #[test]
    fn write_read_cycle_u64() {
        let ram = Ram::new();
        ram.write_double(0, 0xdeadbeef_11223344).expect("written");
        let i = ram.read_double(0).expect("read");

        assert_eq!(i, 0xdeadbeef_11223344, "dead beef");
    }
}
