use crate::irq::Interrupt;

pub trait Device {
    fn write_double(&self, addr: usize, val: u64) -> Result<(), Interrupt>;
    fn write_word(&self, addr: usize, val: u32) -> Result<(), Interrupt>;
    fn write_half(&self, addr: usize, val: u16) -> Result<(), Interrupt>;
    fn write_byte(&self, addr: usize, val: u8) -> Result<(), Interrupt>;
    fn read_double(&self, addr: usize) -> Result<u64, Interrupt>;
    fn read_word(&self, addr: usize) -> Result<u32, Interrupt>;
    fn read_half(&self, addr: usize) -> Result<u16, Interrupt>;
    fn read_byte(&self, addr: usize) -> Result<u8, Interrupt>;
}
