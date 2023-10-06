use std::sync::RwLock;
use std::time::{Duration, Instant};

use crate::bus::Fault;

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

    pub fn write_word(&self, addr: usize, val: u32) -> Result<(), Fault> {
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
            _ => Err(Fault::MemoryFault),
        }
    }

    pub fn read_word(&self, addr: usize) -> Result<u32, Fault> {
        let now = self.start.elapsed();

        match addr {
            MTIMECMP_ADDR => Ok(0xFFFFFFFF),
            MTIMECMP_ADDRH => Ok(0xFFFFFFFF),
            MTIME_ADDR => Ok((now.as_nanos() & 0x0FFFFFFFFu128) as u32),
            MTIME_ADDRH => Ok(((now.as_nanos() >> 32) & 0x0FFFFFFFFu128) as u32),
            _ => Err(Fault::MemoryFault),
        }
    }
}
