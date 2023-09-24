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

impl From<std::num::TryFromIntError> for Error {
    fn from(_value: std::num::TryFromIntError) -> Self {
        Error::InvalidParam
    }
}
impl From<io::Error> for Error {
    fn from(_value: io::Error) -> Self {
        Error::Failed
    }
}

fn sbi_get_spec_version() -> Result<u32, Error> {
    Ok(SBI_VERSION.0 << 24 + SBI_VERSION.1)
}

fn sbi_console_putchar(value: u32) -> Result<u32, Error> {
    let char = [u8::try_from(value)?];
    io::stdout().write_all(&char)?;
    io::stdout().flush()?;
    Ok(0)
}

pub fn call(registers: &mut [u32; 32]) {
    let func = (registers[Register::EID], registers[Register::FID]);

    let result = match func {
        (0x10, 0x0) => sbi_get_spec_version(),
        (0x01, _) => sbi_console_putchar(registers[Register::ARG0]),
        (_, _) => Err(Error::NotSupported),
    };

    match result {
        Ok(value) => {
            registers[Register::ARG0] = Error::Success as u32;
            registers[Register::ARG1] = value;
        }
        Err(error) => {
            eprintln!("error in syscall: {:?}", func);
            registers[Register::ARG0] = error as u32;
            registers[Register::ARG1] = 0;
        }
    }
}
