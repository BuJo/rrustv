use std::fmt::{Display, Formatter};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use log::trace;

use crate::device::Device;
use crate::dynbus::DynBus;
use crate::hart::Hart;
use crate::irq::Interrupt;
use crate::{csr, rtc};

pub const MSIP_HART0_ADDR: usize = 0x0;
pub const MSIP_HART4095_ADDR: usize = 0x3FFC;
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
pub enum InterruptType {
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

impl Display for InterruptType {
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
    fn write_double(&self, addr: usize, val: u64) -> Result<(), Interrupt> {
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

    fn write_word(&self, addr: usize, val: u32) -> Result<(), Interrupt> {
        match addr {
            MSIP_HART0_ADDR..MSIP_HART4095_ADDR => {
                let hartid = (addr - MSIP_HART0_ADDR) / 4;
                if val > 0 {
                    trace!("interrupting hart {} via MIP", hartid);
                }
                Ok(self.msip.store(val > 0, Ordering::Relaxed))
            }
            _ => {
                trace!("writing word to 0x{:x} = {}", addr, val);
                Ok(())
            }
        }
    }

    fn write_half(&self, _addr: usize, _val: u16) -> Result<(), Interrupt> {
        Err(Interrupt::Unimplemented(
            "writing half word unimplemented".into(),
        ))
    }

    fn write_byte(&self, _addr: usize, _val: u8) -> Result<(), Interrupt> {
        Err(Interrupt::Unimplemented(
            "writing byte unimplemented".into(),
        ))
    }

    fn read_double(&self, addr: usize) -> Result<u64, Interrupt> {
        match addr {
            MTIME_ADDR => self.bus.read_double(self.rtc_addr + rtc::MTIME_ADDR),
            _ => {
                trace!("reading double word from 0x{:x}", addr);
                Ok(0)
            }
        }
    }

    fn read_word(&self, addr: usize) -> Result<u32, Interrupt> {
        match addr {
            MSIP_HART0_ADDR..MSIP_HART4095_ADDR => {
                let hartid = (addr - MSIP_HART0_ADDR) / 4;
                trace!("checking if hart {} interrupted via MIP", hartid);
                Ok(self.msip.load(Ordering::Relaxed) as u32)
            }
            MTIME_ADDR => self.bus.read_word(self.rtc_addr + rtc::MTIME_ADDR),
            MTIME_ADDRH => self.bus.read_word(self.rtc_addr + rtc::MTIME_ADDRH),
            _ => {
                trace!("reading word from 0x{:x}", addr);
                Ok(0)
            }
        }
    }

    fn read_half(&self, _addr: usize) -> Result<u16, Interrupt> {
        Err(Interrupt::Unimplemented(
            "reading half word unimplemented".into(),
        ))
    }

    fn read_byte(&self, _addr: usize) -> Result<u8, Interrupt> {
        Err(Interrupt::Unimplemented(
            "reading byte unimplemented".into(),
        ))
    }
}

fn pending_interrupt(mip: u64, mie: u64) -> Option<InterruptType> {
    let ip = mip & mie;

    // External from PLIC
    if ip >> (InterruptType::MEIP as u8) == 0b1 {
        return Some(InterruptType::MEIP);
    }
    if ip >> (InterruptType::SEIP as u8) == 0b1 {
        return Some(InterruptType::SEIP);
    }
    if ip >> (InterruptType::USIP as u8) == 0b1 {
        return Some(InterruptType::USIP);
    }

    // Local Timer
    if ip >> (InterruptType::MTIP as u8) == 0b1 {
        return Some(InterruptType::MTIP);
    }
    if ip >> (InterruptType::STIP as u8) == 0b1 {
        return Some(InterruptType::STIP);
    }
    if ip >> (InterruptType::UEIP as u8) == 0b1 {
        return Some(InterruptType::UEIP);
    }

    // Local Software
    if ip >> (InterruptType::MSIP as u8) == 0b1 {
        return Some(InterruptType::MSIP);
    }
    if ip >> (InterruptType::SSIP as u8) == 0b1 {
        return Some(InterruptType::SSIP);
    }
    if ip >> (InterruptType::USIP as u8) == 0b1 {
        return Some(InterruptType::USIP);
    }

    None
}

pub(crate) fn interrupt<BT: Device>(hart: &Hart<BT>) -> Option<InterruptType> {
    let mode = PrivilegeLevel::M;
    let mstatus = hart.get_csr(csr::MSTATUS);

    let enabled = match mode {
        PrivilegeLevel::M => mstatus & 0b0001 > 0,
        PrivilegeLevel::S => mstatus & 0b0010 > 0,
        PrivilegeLevel::U => mstatus & 0b1001 > 0,
    };

    if !enabled {
        return None;
    }

    let msip = hart.bus.read_word(0x2000000 + MSIP_HART0_ADDR).expect(".") as u64; // XXX: bad.
    let mip = hart.get_csr(csr::MIP);
    let mie = hart.get_csr(csr::MIE);

    let pending = pending_interrupt(mip | msip, mie);

    pending
}
