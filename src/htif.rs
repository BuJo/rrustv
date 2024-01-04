use crate::device::Device;
use crate::irq::Interrupt;
use crate::irq::Interrupt::{Halt, MemoryFault, Unaligned};

pub struct Htif {}

impl Htif {
    pub fn new() -> Htif {
        Htif {}
    }
}

impl Default for Htif {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for Htif {
    fn write_double(&self, addr: usize, _val: u64) -> Result<(), Interrupt> {
        match addr {
            0x0 => Err(Halt),
            _ => Err(MemoryFault(addr)),
        }
    }

    fn write_word(&self, addr: usize, _val: u32) -> Result<(), Interrupt> {
        match addr {
            0x0 => Err(Halt),
            _ => Err(MemoryFault(addr)),
        }
    }

    fn write_half(&self, addr: usize, _val: u16) -> Result<(), Interrupt> {
        Err(Unaligned(addr))
    }

    fn write_byte(&self, addr: usize, _val: u8) -> Result<(), Interrupt> {
        Err(Unaligned(addr))
    }

    fn read_double(&self, addr: usize) -> Result<u64, Interrupt> {
        Err(MemoryFault(addr))
    }

    fn read_word(&self, addr: usize) -> Result<u32, Interrupt> {
        Err(MemoryFault(addr))
    }

    fn read_half(&self, addr: usize) -> Result<u16, Interrupt> {
        Err(Unaligned(addr))
    }

    fn read_byte(&self, addr: usize) -> Result<u8, Interrupt> {
        Err(Unaligned(addr))
    }
}
