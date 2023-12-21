use crate::device::Device;
use crate::ins::Instruction;
use log::trace;

#[derive(Debug)]
pub enum Fault {
    MemoryFault(usize),
    Unmapped(usize),
    Unaligned(usize),
    Halt,
    Unimplemented(String),
    InstructionDecodingError,
    IllegalOpcode(Instruction),
}

pub struct Plic {}
impl Plic {
    pub fn new() -> Plic {
        Plic {}
    }
}
impl Device for Plic {
    fn write_double(&self, addr: usize, val: u64) -> Result<(), Fault> {
        trace!("plic: writing to {} = {}", addr, val);
        Ok(())
    }

    fn write_word(&self, addr: usize, val: u32) -> Result<(), Fault> {
        trace!("plic: writing to {} = {}", addr, val);
        Ok(())
    }

    fn write_half(&self, _addr: usize, _val: u16) -> Result<(), Fault> {
        Err(Fault::Unimplemented(
            "plic: writing half word unimplemented".into(),
        ))
    }

    fn write_byte(&self, _addr: usize, _val: u8) -> Result<(), Fault> {
        Err(Fault::Unimplemented(
            "plic: writing byte unimplemented".into(),
        ))
    }

    fn read_double(&self, addr: usize) -> Result<u64, Fault> {
        trace!("plic: reading from {}", addr);
        Ok(0)
    }

    fn read_word(&self, addr: usize) -> Result<u32, Fault> {
        trace!("plic: reading from {}", addr);
        Ok(0)
    }

    fn read_half(&self, _addr: usize) -> Result<u16, Fault> {
        Err(Fault::Unimplemented(
            "plic: reading half word unimplemented".into(),
        ))
    }

    fn read_byte(&self, _addr: usize) -> Result<u8, Fault> {
        Err(Fault::Unimplemented(
            "plic: reading byte unimplemented".into(),
        ))
    }
}
