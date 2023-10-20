use crate::plic::Fault;

pub trait Device {
    fn write_double(&self, addr: usize, val: u64) -> Result<(), Fault>;
    fn write_word(&self, addr: usize, val: u32) -> Result<(), Fault>;
    fn write_half(&self, addr: usize, val: u16) -> Result<(), Fault>;
    fn write_byte(&self, addr: usize, val: u8) -> Result<(), Fault>;
    fn read_double(&self, addr: usize) -> Result<u64, Fault>;
    fn read_word(&self, addr: usize) -> Result<u32, Fault>;
    fn read_half(&self, addr: usize) -> Result<u16, Fault>;
    fn read_byte(&self, addr: usize) -> Result<u8, Fault>;
}
