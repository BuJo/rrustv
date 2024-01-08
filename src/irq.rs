use crate::ins::Instruction;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Interrupt {
    MemoryFault(usize),
    Unmapped(usize),
    Unaligned(usize),
    Halt,
    Unimplemented(String),
    InstructionDecodingError,
    IllegalOpcode(Instruction),
}

impl Display for Interrupt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "some error")
    }
}

impl std::error::Error for Interrupt {}
