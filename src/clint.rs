use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use log::trace;

use crate::csr::Csr;
use crate::device::Device;
use crate::dynbus::DynBus;
use crate::plic::Fault;
use crate::{csr, rtc};

pub const MSIP_HART0_ADDR: usize = 0x0;
pub const MTIME_ADDR: usize = 0xbff8;
pub const MTIME_ADDRH: usize = 0xbffc;
pub const MTIMECMP_ADDR: usize = 0x4000;

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

pub struct Clint {
    bus: Arc<DynBus>,
    rtc_addr: usize,
    msip: AtomicBool, // XXX: only one hart
}

impl Clint {
    pub fn new(bus: Arc<DynBus>, rtc_addr: usize) -> Clint {
        Clint {
            bus,
            rtc_addr,
            msip: AtomicBool::new(false),
        }
    }
}

impl Device for Clint {
    fn write_double(&self, addr: usize, val: u64) -> Result<(), Fault> {
        match addr {
            MTIMECMP_ADDR => self
                .bus
                .write_double(self.rtc_addr + rtc::MTIMECMP_ADDR, val),
            _ => {
                trace!("writing double word to 0x{:x} = {}", addr, val);
                Ok(())
            }
        }
    }

    fn write_word(&self, addr: usize, val: u32) -> Result<(), Fault> {
        match addr {
            MSIP_HART0_ADDR => Ok(self.msip.store(val > 0, Ordering::Relaxed)),
            _ => {
                trace!("writing word to 0x{:x} = {}", addr, val);
                Ok(())
            }
        }
    }

    fn write_half(&self, _addr: usize, _val: u16) -> Result<(), Fault> {
        Err(Fault::Unimplemented(
            "writing half word unimplemented".into(),
        ))
    }

    fn write_byte(&self, _addr: usize, _val: u8) -> Result<(), Fault> {
        Err(Fault::Unimplemented(
            "writing byte unimplemented".into(),
        ))
    }

    fn read_double(&self, addr: usize) -> Result<u64, Fault> {
        match addr {
            MTIME_ADDR => self.bus.read_double(self.rtc_addr + rtc::MTIME_ADDR),
            _ => {
                trace!("reading double word from 0x{:x}", addr);
                Ok(0)
            }
        }
    }

    fn read_word(&self, addr: usize) -> Result<u32, Fault> {
        match addr {
            MSIP_HART0_ADDR => Ok(self.msip.load(Ordering::Relaxed) as u32),
            MTIME_ADDR => self.bus.read_word(self.rtc_addr + rtc::MTIME_ADDR),
            MTIME_ADDRH => self.bus.read_word(self.rtc_addr + rtc::MTIME_ADDRH),
            _ => {
                trace!("reading word from 0x{:x}", addr);
                Ok(0)
            }
        }
    }

    fn read_half(&self, _addr: usize) -> Result<u16, Fault> {
        Err(Fault::Unimplemented(
            "reading half word unimplemented".into(),
        ))
    }

    fn read_byte(&self, _addr: usize) -> Result<u8, Fault> {
        Err(Fault::Unimplemented(
            "reading byte unimplemented".into(),
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
