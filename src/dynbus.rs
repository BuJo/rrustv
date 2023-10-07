use std::ops::Range;

use crate::device::Device;
use crate::htif::Htif;
use crate::plic::Fault;
use crate::plic::Fault::{Halt, MemoryFault};
use crate::ram::Ram;
use crate::rom::Rom;

pub struct DynBus {
    devices: Vec<(Range<usize>, Box<dyn Device>)>,
}

impl DynBus {
    pub fn new() -> DynBus {
        Self { devices: vec![] }
    }

    pub fn map(&mut self, device: impl Device + 'static, range: Range<usize>) {
        self.devices.push((range, Box::new(device)));
    }
}

impl Device for DynBus {
    fn write_word(&self, addr: usize, val: u32) -> Result<(), Fault> {
        for (range, device) in &self.devices {
            if range.contains(&addr) {
                return device.write_word(addr - range.start, val);
            }
        }
        Err(MemoryFault(addr))
    }

    fn write_byte(&self, addr: usize, val: u8) -> Result<(), Fault> {
        for (range, device) in &self.devices {
            if range.contains(&addr) {
                return device.write_byte(addr - range.start, val);
            }
        }
        Err(MemoryFault(addr))
    }

    fn read_word(&self, addr: usize) -> Result<u32, Fault> {
        for (range, device) in &self.devices {
            if range.contains(&addr) {
                return device.read_word(addr - range.start);
            }
        }
        Err(MemoryFault(addr))
    }

    fn read_byte(&self, addr: usize) -> Result<u8, Fault> {
        for (range, device) in &self.devices {
            if range.contains(&addr) {
                return device.read_byte(addr - range.start);
            }
        }
        Err(MemoryFault(addr))
    }
}

impl Device for Ram {
    fn write_word(&self, addr: usize, val: u32) -> Result<(), Fault> {
        self.write_word(addr, val).ok_or(MemoryFault(addr))
    }

    fn write_byte(&self, addr: usize, val: u8) -> Result<(), Fault> {
        self.write_byte(addr, val).ok_or(MemoryFault(addr))
    }

    fn read_word(&self, addr: usize) -> Result<u32, Fault> {
        self.read_word(addr).ok_or(MemoryFault(addr))
    }

    fn read_byte(&self, addr: usize) -> Result<u8, Fault> {
        self.read_byte(addr).ok_or(MemoryFault(addr))
    }
}

impl Device for Rom {
    fn write_word(&self, addr: usize, _val: u32) -> Result<(), Fault> {
        Err(MemoryFault(addr))
    }

    fn write_byte(&self, addr: usize, _val: u8) -> Result<(), Fault> {
        Err(MemoryFault(addr))
    }

    fn read_word(&self, addr: usize) -> Result<u32, Fault> {
        self.read_word(addr).ok_or(MemoryFault(addr))
    }

    fn read_byte(&self, addr: usize) -> Result<u8, Fault> {
        self.read_byte(addr).ok_or(MemoryFault(addr))
    }
}


impl Device for Htif {
    fn write_word(&self, _addr: usize, _val: u32) -> Result<(), Fault> {
        Err(Halt)
    }

    fn write_byte(&self, _addr: usize, _val: u8) -> Result<(), Fault> {
        Err(Halt)
    }

    fn read_word(&self, _addr: usize) -> Result<u32, Fault> {
        Err(Halt)
    }

    fn read_byte(&self, _addr: usize) -> Result<u8, Fault> {
        Err(Halt)
    }
}

mod test {
    use crate::device::Device;
    use crate::dynbus::DynBus;
    use crate::htif::Htif;
    use crate::ram::Ram;

    #[test]
    fn basic() {
        let bus = DynBus::new();
        let err = bus.write_word(0x0, 0x0);
        assert_eq!(err.is_ok(), false, "no device should error on write");
    }

    #[test]
    fn ram() {
        let ram = Ram::new();
        let mut bus = DynBus::new();
        bus.map(ram, 0..0x2000);

        let err = bus.write_word(0x0, 0x0);
        assert_eq!(err.is_ok(), true, "ram should write");
    }


    #[test]
    fn htif() {
        let htif = Htif::new();
        let mut bus = DynBus::new();
        bus.map(htif, 0..50);

        let err = bus.write_word(0x0, 0x0);
        assert_eq!(err.is_ok(), false, "should shut down");
    }
}
