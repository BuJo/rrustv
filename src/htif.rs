use crate::device::Device;
use crate::plic::Fault;
use crate::plic::Fault::{Halt, MemoryFault};

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
    fn write_word(&self, addr: usize, _val: u32) -> Result<(), Fault> {
        match addr {
            0x0 => Err(Halt),
            _ => Err(MemoryFault(addr)),
        }
    }

    fn write_byte(&self, addr: usize, _val: u8) -> Result<(), Fault> {
        Err(MemoryFault(addr))
    }

    fn read_word(&self, addr: usize) -> Result<u32, Fault> {
        Err(MemoryFault(addr))
    }

    fn read_byte(&self, addr: usize) -> Result<u8, Fault> {
        Err(MemoryFault(addr))
    }
}
