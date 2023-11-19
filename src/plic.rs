use std::fmt::{Display, Formatter};
use crate::ins::Instruction;

#[derive(Debug)]
pub enum Fault {
    MemoryFault(usize),
    Unaligned(usize),
    Halt,
    Unimplemented,
    InstructionDecodingError,
    IllegalOpcode(Instruction),
}

impl Display for Fault {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Fault::MemoryFault(addr) => write!(f, "memory fault at {:x}", addr),
            Fault::Unaligned(addr) => write!(f, "unaligned access at {:x}", addr),
            Fault::Halt =>  write!(f, "halted"),
            Fault::Unimplemented => write!(f, "unimplemented"),
            Fault::InstructionDecodingError => write!(f, "failed decoding instruction"),
            Fault::IllegalOpcode(ins) => write!(f, "illegal instruction: {:?}", ins),
        }
    }
}
