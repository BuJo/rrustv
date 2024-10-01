use std::fmt::{Display, Formatter};

use crate::ins::Instruction;

#[derive(Debug, Clone)]
pub enum Interrupt {
    MemoryFault(usize),
    Unmapped(usize),
    Unaligned(usize),
    Halt,
    Unimplemented(String),
    InstructionDecodingError(Instruction),
    IllegalOpcode(Instruction),
}

impl Display for Interrupt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "some error")
    }
}

impl std::error::Error for Interrupt {}

#[allow(non_camel_case_types, unused)]
#[repr(u64)]
pub(crate) enum Mcause {
    INS_MISALIGNED = 0x0,
    INS_ACCESS = 0x1,
    INS_ILL = 0x2,
    BREAKPOINT = 0x3,
    LOAD_MISALIGNED = 0x4,
    LOAD_ACCESS = 0x5,
    STORE_MISALIGNED = 0x6,
    STORE_ACCESS = 0x7,
    UECALL = 0x8,
    SECALL = 0x9,
    HECALL = 0xa,
    MECALL = 0xb,
    USIP = 0x8000000000000000,
    SSIP = 0x8000000000000001,
    HSIP = 0x8000000000000002,
    MSIP = 0x8000000000000003, // Machine software interrupt
    UTIP = 0x8000000000000004,
    STIP = 0x8000000000000005,
    HTIP = 0x8000000000000006,
    MTIP = 0x8000000000000007, // Machine timer interrupt
    UEIP = 0x8000000000000008,
    SEIP = 0x8000000000000009,
    HEIP = 0x800000000000000a,
    MEIP = 0x800000000000000b, // Machine external interrupt
}
