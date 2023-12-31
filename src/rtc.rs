use std::sync::RwLock;
use std::time::{Duration, Instant};

use crate::device::Device;
use crate::plic::Fault;

pub const MTIMECMP_ADDR: usize = 0x4000;
pub const MTIMECMP_ADDRH: usize = 0x4004;
pub const MTIME_ADDR: usize = 0x4008;
pub const MTIME_ADDRH: usize = 0x400c;

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
}

impl Default for Rtc {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for Rtc {
    fn write_double(&self, addr: usize, val: u64) -> Result<(), Fault> {
        match addr {
            MTIMECMP_ADDR => {
                let mut v = self.mtimecmptmp.write().unwrap();
                *v = val;
                Ok(())
            }
            _ => Err(Fault::MemoryFault(addr)),
        }
    }

    fn write_word(&self, addr: usize, val: u32) -> Result<(), Fault> {
        match addr {
            MTIMECMP_ADDR => {
                let mut low = self.mtimecmptmp.write().unwrap();
                *low = (*low & 0xFFFF_FFFF_0000_0000) | val as u64;
                Ok(())
            }
            MTIMECMP_ADDRH => {
                let mut high = self.mtimecmptmp.write().unwrap();
                *high = (*high & 0x0000_0000_FFFF_FFFF) | ((val as u64) << 32);

                let mut shared = self.mtimecmp.write().unwrap();
                *shared = Duration::from_nanos(*high);
                Ok(())
            }
            _ => Err(Fault::MemoryFault(addr)),
        }
    }

    fn write_half(&self, addr: usize, _val: u16) -> Result<(), Fault> {
        Err(Fault::Unaligned(addr))
    }

    fn write_byte(&self, addr: usize, _val: u8) -> Result<(), Fault> {
        Err(Fault::Unaligned(addr))
    }

    fn read_double(&self, addr: usize) -> Result<u64, Fault> {
        let now = self.start.elapsed();

        match addr {
            MTIMECMP_ADDR => Ok(0xFFFFFFFF),
            MTIME_ADDR => Ok(now.as_nanos() as u64),
            _ => Err(Fault::MemoryFault(addr)),
        }
    }

    fn read_word(&self, addr: usize) -> Result<u32, Fault> {
        let now = self.start.elapsed();

        match addr {
            MTIMECMP_ADDR => Ok(0xFFFFFFFF),
            MTIMECMP_ADDRH => Ok(0xFFFFFFFF),
            MTIME_ADDR => Ok((now.as_nanos() & 0x0FFFFFFFFu128) as u32),
            MTIME_ADDRH => Ok(((now.as_nanos() >> 32) & 0x0FFFFFFFFu128) as u32),
            _ => Err(Fault::MemoryFault(addr)),
        }
    }

    fn read_half(&self, addr: usize) -> Result<u16, Fault> {
        Err(Fault::Unaligned(addr))
    }

    fn read_byte(&self, addr: usize) -> Result<u8, Fault> {
        Err(Fault::Unaligned(addr))
    }
}
