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

        shared.splice(addr..(addr + 8), val.to_le_bytes());

        Ok(())
    }
    fn write_word(&self, addr: usize, val: u32) -> Result<(), Fault> {
        let mut shared = self.data.write().unwrap();

        shared.splice(addr..(addr + 4), val.to_le_bytes());

        Ok(())
    }

    fn write_half(&self, addr: usize, val: u16) -> Result<(), Fault> {
        let mut shared = self.data.write().unwrap();

        shared.splice(addr..(addr + 2), val.to_le_bytes());

        Ok(())
    }

    fn write_byte(&self, addr: usize, val: u8) -> Result<(), Fault> {
        let mut shared = self.data.write().unwrap();

        *(shared.get_mut(addr).ok_or(MemoryFault(addr))?) = val;
        Ok(())
    }

    fn read_double(&self, addr: usize) -> Result<u64, Fault> {
        let data = self.data.read().unwrap();

        let bytes = data.get(addr..(addr + 8)).ok_or(MemoryFault(addr))?;
        let bytes = <[u8; 8]>::try_from(bytes).map_err(|_| MemoryFault(addr))?;

        let val = u64::from_le_bytes(bytes);
        Ok(val)
    }
    fn read_word(&self, addr: usize) -> Result<u32, Fault> {
        let data = self.data.read().unwrap();

        let bytes = data.get(addr..(addr + 4)).ok_or(MemoryFault(addr))?;
        let bytes = <[u8; 4]>::try_from(bytes).map_err(|_| MemoryFault(addr))?;
        let val = u32::from_le_bytes(bytes);
        Ok(val)
    }

    fn read_half(&self, addr: usize) -> Result<u16, Fault> {
        let data = self.data.read().unwrap();

        let bytes = data.get(addr..(addr + 2)).ok_or(MemoryFault(addr))?;
        let bytes = <[u8; 2]>::try_from(bytes).map_err(|_| MemoryFault(addr))?;
        let val = u16::from_le_bytes(bytes);
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
