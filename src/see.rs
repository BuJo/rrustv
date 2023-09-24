// Supervisor Execution Environment (SEE) implementing
// RISC-V SBI (Supervisor Binary Interface)
use std::io::{self, Write};
use std::ops::{Index, IndexMut};

const SBI_VERSION: (u32, u32) = (1, 0);

#[allow(dead_code)]
enum Register {
    // a0: (Error Code)
    ARG0 = 10,
    // a1 (Value)
    ARG1 = 11,
    // FID: a6 (Function ID)
    FID = 16,
    // EID: a7 (Extension ID)
    EID = 17,
}

impl Index<Register> for [u32; 32] {
    type Output = u32;

    fn index(&self, idx: Register) -> &Self::Output {
        &self[idx as usize]
    }
}
impl IndexMut<Register> for [u32; 32] {
    fn index_mut(&mut self, idx: Register) -> &mut Self::Output {
        &mut self[idx as usize]
    }
}

#[allow(dead_code)]
enum Error {
    Success = 0,
    Failed = -1,
    NotSupported = -2,
    InvalidParam = -3,
    Denied = -4,
    InvalidAddress = -5,
    AlreadyAvailable = -6,
    AlreadyStarted = -7,
    AlreadyStopped = -8,
}

pub fn call(registers: &mut [u32; 32]) {
    let func = (registers[Register::EID], registers[Register::FID]);

    match func {
        (0x10, 0x0) => {
            let spec_version: u32 = SBI_VERSION.0 << 24 + SBI_VERSION.1;
            registers[Register::ARG0] = Error::Success as u32;
            registers[Register::ARG1] = spec_version;
        }
        (0x01, _) => {
            print!("{}", char::from_u32(registers[Register::ARG0]).unwrap());
            io::stdout().flush().unwrap();
        }
        (eid, fid) => {
            registers[Register::ARG0] = Error::NotSupported as u32;
            eprintln!("invalid syscall: {}/{}", eid, fid)
        }
    }
}
