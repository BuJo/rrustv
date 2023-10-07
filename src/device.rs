use crate::plic::Fault;

pub trait Device {
    fn write_word(&self, addr: usize, val: u32) -> Result<(), Fault>;
    fn write_byte(&self, addr: usize, val: u8) -> Result<(), Fault>;
    fn read_word(&self, addr: usize) -> Result<u32, Fault>;
    fn read_byte(&self, addr: usize) -> Result<u8, Fault>;
}
