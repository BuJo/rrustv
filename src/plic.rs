use crate::ins::Instruction;

#[derive(Debug)]
pub enum Fault {
    MemoryFault(usize),
    Unmapped(usize),
    Unaligned(usize),
    Halt,
    Unimplemented,
    InstructionDecodingError,
    IllegalOpcode(Instruction),
}
