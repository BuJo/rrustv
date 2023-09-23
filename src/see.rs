// Supervisor Execution Environment (SEE) implementing
// RISC-V SBI (Supervisor Binary Interface)

use crate::see::Error::Success;

const SBI_VERSION: (u32, u32) = (1, 0);

pub const SBI_ARG0_REG: usize = 10; // a0 (Error Code)
pub const SBI_ARG1_REG: usize = 11; // a1 (Value)
pub const SBI_FUNCTION_REG: usize = 17; // FID: a6 (Function ID)
pub const SBI_SYSCALL_REG: usize = 17; // EID: a7 (Extension ID)

#[allow(dead_code)]
pub enum Error {
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

    let func = (registers[SBI_SYSCALL_REG], registers[SBI_FUNCTION_REG]);

    match func {
        (0x10, 0x0) => {
            let spec_version: u32 = SBI_VERSION.0 << 24 + SBI_VERSION.1;
            registers[SBI_ARG0_REG] = Success as u32;
            registers[SBI_ARG1_REG] = spec_version;
        }
        (0x01, _) => {
            print!("{}", char::from_u32(registers[SBI_ARG0_REG]).unwrap())
        }
        (_, _) => {
            registers[SBI_ARG0_REG] = Error::NotSupported as u32;
            println!("invalid syscall: {}", registers[SBI_SYSCALL_REG])
        }
    }
}
