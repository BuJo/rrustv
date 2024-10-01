use std::sync::RwLock;
use std::time::{Duration, Instant};

use log::trace;

use crate::device::Device;
use crate::irq::Interrupt;

pub const MTIMECMP_ADDR: usize = 0x0;
pub const MTIMECMP_ADDRH: usize = 0x4;
pub const MTIME_ADDR: usize = 0x8;
pub const MTIME_ADDRH: usize = 0xc;

pub struct Rtc {
    start: Instant,
    mtimecmp: RwLock<Duration>,
    mtimecmptmp: RwLock<u64>,
}

impl Rtc {
    pub fn new() -> Rtc {
        Self {
            start: Instant::now(),
            mtimecmp: RwLock::new(Duration::MAX),
            mtimecmptmp: RwLock::new(u64::MAX),
        }
    }

    fn get_time(&self) -> u64 {
        (self.start.elapsed().as_nanos() & 0xFFFF_FFFF_FFFF_FFFF) as u64
    }

    fn set_timer(&self, val: u64) {
        let mut mtimecmp = self.mtimecmp.write().unwrap();
        *mtimecmp = Duration::from_nanos(val);
        trace!("setting timer to: {:?}", *mtimecmp)
    }

    fn get_timer(&self) -> u64 {
        let mtimecmp = self.mtimecmp.read().unwrap();
        mtimecmp.as_nanos() as u64
    }
}

impl Default for Rtc {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for Rtc {
    fn write_double(&self, addr: usize, val: u64) -> Result<(), Interrupt> {
        match addr {
            MTIMECMP_ADDR => {
                self.set_timer(val);
                Ok(())
            }
            _ => Err(Interrupt::MemoryFault(addr)),
        }
    }

    fn write_word(&self, addr: usize, val: u32) -> Result<(), Interrupt> {
        match addr {
            MTIMECMP_ADDR => {
                let mut tmp = self.mtimecmptmp.write().unwrap();
                *tmp = val as u64;
                Ok(())
            }
            MTIMECMP_ADDRH => {
                let tmp = self.mtimecmptmp.write().unwrap();
                let time = (*tmp & 0x0000_0000_FFFF_FFFF) | ((val as u64) << 32);

                self.set_timer(time);
                Ok(())
            }
            _ => Err(Interrupt::MemoryFault(addr)),
        }
    }

    fn write_half(&self, addr: usize, _val: u16) -> Result<(), Interrupt> {
        Err(Interrupt::Unaligned(addr))
    }

    fn write_byte(&self, addr: usize, _val: u8) -> Result<(), Interrupt> {
        Err(Interrupt::Unaligned(addr))
    }

    fn read_double(&self, addr: usize) -> Result<u64, Interrupt> {
        match addr {
            MTIMECMP_ADDR => Ok(self.get_timer()),
            MTIME_ADDR => Ok(self.get_time()),
            _ => Err(Interrupt::MemoryFault(addr)),
        }
    }

    fn read_word(&self, addr: usize) -> Result<u32, Interrupt> {
        match addr {
            MTIMECMP_ADDR => Ok((self.get_timer() & 0xFFFFFFFF) as u32),
            MTIMECMP_ADDRH => Ok(((self.get_timer() >> 32) & 0xFFFFFFFF) as u32),
            MTIME_ADDR => Ok((self.get_time() & 0xFFFFFFFF) as u32),
            MTIME_ADDRH => Ok(((self.get_time() >> 32) & 0xFFFFFFFF) as u32),
            _ => Err(Interrupt::MemoryFault(addr)),
        }
    }

    fn read_half(&self, addr: usize) -> Result<u16, Interrupt> {
        Err(Interrupt::Unaligned(addr))
    }

    fn read_byte(&self, addr: usize) -> Result<u8, Interrupt> {
        Err(Interrupt::Unaligned(addr))
    }
}
