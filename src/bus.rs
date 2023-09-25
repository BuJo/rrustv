use crate::bus::Fault::MemoryFault;
use crate::ram::Ram;

pub struct Bus {
    rom: Ram,
}

#[derive(Debug)]
pub enum Fault {
    MemoryFault,
}

impl Bus {
    pub fn new(rom: Ram) -> Bus {
        Self { rom }
    }

    pub fn write_word(&self, addr: usize, val: u32) -> Result<(), Fault> {
        match addr {
            0x0000..=0x2000 => Ok(self.rom.write_word(addr, val)),
            _ => Err(MemoryFault),
        }
    }

    pub fn write_byte(&self, addr: usize, val: u8) -> Result<(), Fault> {
        match addr {
            0x0000..=0x2000 => Ok(self.rom.write_byte(addr, val)),
            _ => Err(MemoryFault),
        }
    }

    pub fn read_word(&self, addr: usize) -> Result<u32, Fault> {
        match addr {
            0x0000..=0x2000 => Ok(self.rom.read_word(addr)),
            _ => Err(MemoryFault),
        }
    }

    pub fn read_byte(&self, addr: usize) -> Result<u8, Fault> {
        match addr {
            0x0000..=0x2000 => Ok(self.rom.read_byte(addr)),
            _ => Err(MemoryFault),
        }
    }
}
