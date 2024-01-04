use crate::ins::Instruction;

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
