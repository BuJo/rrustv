use crate::csr;
use crate::csr::Csr;
use crate::device::Device;
use crate::plic::Fault;
use log::trace;
use std::fmt::{Display, Formatter};

#[allow(unused)]
enum PrivilegeLevel {
    M,
    S,
    U,
}

#[derive(Debug)]
pub enum Interrupt {
    // External from PLIC
    MEIP = 11,
    SEIP = 9,
    UEIP = 8,

    // Local Timer
    MTIP = 7,
    STIP = 5,
    UTIP = 4,

    // Local Software
    MSIP = 3,
    SSIP = 1,
    USIP = 0,
}

impl Display for Interrupt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "interrupt: {:?}", self)
    }
}

pub struct Clint {}
impl Clint {
    pub fn new() -> Clint {
        Clint {}
    }
}

impl Device for Clint {
    fn write_double(&self, addr: usize, val: u64) -> Result<(), Fault> {
        trace!("clint: writing to {} = {}", addr, val);
        Ok(())
    }

    fn write_word(&self, addr: usize, val: u32) -> Result<(), Fault> {
        trace!("clint: writing to {} = {}", addr, val);
        Ok(())
    }

    fn write_half(&self, _addr: usize, _val: u16) -> Result<(), Fault> {
        Err(Fault::Unimplemented(
            "clint: writing half word unimplemented".into(),
        ))
    }

    fn write_byte(&self, _addr: usize, _val: u8) -> Result<(), Fault> {
        Err(Fault::Unimplemented(
            "clint: writing byte unimplemented".into(),
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
            "clint: reading half word unimplemented".into(),
        ))
    }

    fn read_byte(&self, _addr: usize) -> Result<u8, Fault> {
        Err(Fault::Unimplemented(
            "clint: reading byte unimplemented".into(),
        ))
    }
}

fn pending_interrupt(mip: u64, mie: u64) -> Option<Interrupt> {
    let ip = mip & mie;

    // External from PLIC
    if ip >> (Interrupt::MEIP as u8) == 0b1 {
        return Some(Interrupt::MEIP);
    }
    if ip >> (Interrupt::SEIP as u8) == 0b1 {
        return Some(Interrupt::SEIP);
    }
    if ip >> (Interrupt::USIP as u8) == 0b1 {
        return Some(Interrupt::USIP);
    }

    // Local Timer
    if ip >> (Interrupt::MTIP as u8) == 0b1 {
        return Some(Interrupt::MTIP);
    }
    if ip >> (Interrupt::STIP as u8) == 0b1 {
        return Some(Interrupt::STIP);
    }
    if ip >> (Interrupt::UEIP as u8) == 0b1 {
        return Some(Interrupt::UEIP);
    }

    // Local Software
    if ip >> (Interrupt::MSIP as u8) == 0b1 {
        return Some(Interrupt::MSIP);
    }
    if ip >> (Interrupt::SSIP as u8) == 0b1 {
        return Some(Interrupt::SSIP);
    }
    if ip >> (Interrupt::USIP as u8) == 0b1 {
        return Some(Interrupt::USIP);
    }

    None
}

pub(crate) fn interrupt(csr: &Csr) -> Option<Interrupt> {
    let mode = PrivilegeLevel::M;
    let mstatus = csr.read(csr::MSTATUS);

    let enabled = match mode {
        PrivilegeLevel::M => mstatus & 0b0001 > 0,
        PrivilegeLevel::S => mstatus & 0b0010 > 0,
        PrivilegeLevel::U => mstatus & 0b1001 > 0,
    };

    if !enabled {
        return None;
    }

    let mip = csr.read(csr::MIP);
    let mie = csr.read(csr::MIE);

    let pending = pending_interrupt(mip, mie);

    pending
}
