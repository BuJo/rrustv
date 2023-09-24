// Supervisor Execution Environment (SEE) implementing
// RISC-V SBI (Supervisor Binary Interface)
use std::io::{self, Write};
use std::ops::{Index, IndexMut};

const SBI_VERSION: (u32, u32) = (1, 0);
const SBI_IMPL_ID: u32 = 0xFFFFFFFF;
const SBI_IMPL_VERSION: u32 = 1;

#[allow(dead_code)]
enum Register {
    // a0: in/out (Error Code)
    ARG0 = 10,
    // a1: in/out (Value)
    ARG1 = 11,
    // a6: FID (Function ID)
    FID = 16,
    // a7: EID (Extension ID)
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

// Base Extension (EID #0x10)

fn sbi_get_spec_version() -> Result<u32, Error> {
    Ok(SBI_VERSION.0 << 24 + SBI_VERSION.1)
}

fn sbi_get_sbi_impl_id() -> Result<u32, Error> {
    Ok(SBI_IMPL_ID)
}

fn sbi_get_sbi_impl_version() -> Result<u32, Error> {
    Ok(SBI_IMPL_VERSION)
}

fn sbi_probe_extension(extension_id: u32) -> Result<u32, Error> {
    match extension_id {
        0x01 => Ok(1),
        0x10 => Ok(1),
        _ => Ok(0)
    }
}

fn sbi_get_mvendorid() -> Result<u32, Error> {
    Ok(0)
}

fn sbi_get_marchid() -> Result<u32, Error> {
    Ok(1)
}

fn sbi_get_mimpid() -> Result<u32, Error> {
    Ok(SBI_IMPL_VERSION)
}

//  Legacy Extensions (EIDs #0x00 - #0x0F)

fn sbi_console_putchar(value: u32) -> Result<u32, Error> {
    let char = [u8::try_from(value)?];
    io::stdout().write_all(&char)?;
    io::stdout().flush()?;
    Ok(0)
}

// Legacy Extensions have a different calling convention
fn call_0_1(registers: &mut [u32; 32]) {
    let func = registers[Register::EID];

    let result = match func {
        0x01 => sbi_console_putchar(registers[Register::ARG0]),
        _ => Err(Error::NotSupported),
    };

    match result {
        Ok(value) => {
            registers[Register::ARG0] = value;
        }
        Err(error) => {
            eprintln!("error in syscall: {:?}", func);
            registers[Register::ARG0] = error as u32;
        }
    }
}

fn call_0_2(registers: &mut [u32; 32]) {
    let func = (registers[Register::EID], registers[Register::FID]);

    let result = match func {
        (0x10, 0x0) => sbi_get_spec_version(),
        (0x10, 0x1) => sbi_get_sbi_impl_id(),
        (0x10, 0x2) => sbi_get_sbi_impl_version(),
        (0x10, 0x3) => sbi_probe_extension(registers[Register::ARG0]),
        (0x10, 0x4) => sbi_get_mvendorid(),
        (0x10, 0x5) => sbi_get_marchid(),
        (0x10, 0x6) => sbi_get_mimpid(),
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
        }
    }
}

pub fn call(registers: &mut [u32; 32]) {
    if (0x00..=0x0F).contains(&registers[Register::EID]) {
        call_0_1(registers)
    } else {
        call_0_2(registers)
    }
}
