use std::ops::Range;
use std::sync::RwLock;

use crate::device::Device;
use crate::plic::Fault;
use crate::plic::Fault::MemoryFault;

type DeviceList = Vec<(Range<usize>, Box<dyn Device>)>;

pub struct DynBus {
    devices: RwLock<DeviceList>,
}

// Safety: Every interaction is gated through the RwLock protecting the devices
// additionally the bus should not change while the machine is running?  Hot plugging
// RAM or CPUs should be incredibly rare...
unsafe impl Send for DynBus {}

unsafe impl Sync for DynBus {}

impl DynBus {
    pub fn new() -> DynBus {
        Self {
            devices: RwLock::new(vec![]),
        }
    }

    pub fn map(&mut self, device: impl Device + 'static, range: Range<usize>) {
        let mut devices = self.devices.write().unwrap();

        devices.push((range, Box::new(device)));
    }

    pub fn read(&self, addr: usize, data: &mut [u8]) -> Result<(), Fault> {
        for i in 0..data.len() {
            data[i] = self.read_byte(addr + i)?
        }
        Ok(())
    }
    pub fn write(&self, addr: usize, data: &[u8]) -> Result<(), Fault> {
        for i in 0..data.len() {
            self.write_byte(addr + i, data[i])?
        }
        Ok(())
    }
}

impl Default for DynBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for DynBus {
    fn write_double(&self, addr: usize, val: u64) -> Result<(), Fault> {
        let devices = self.devices.read().unwrap();

        for (range, device) in devices.iter() {
            if range.contains(&addr) {
                return device.write_double(addr - range.start, val);
            }
        }
        Err(MemoryFault(addr))
    }
    fn write_word(&self, addr: usize, val: u32) -> Result<(), Fault> {
        let devices = self.devices.read().unwrap();

        for (range, device) in devices.iter() {
            if range.contains(&addr) {
                return device.write_word(addr - range.start, val);
            }
        }
        Err(MemoryFault(addr))
    }

    fn write_half(&self, addr: usize, val: u16) -> Result<(), Fault> {
        let devices = self.devices.read().unwrap();

        for (range, device) in devices.iter() {
            if range.contains(&addr) {
                return device.write_half(addr - range.start, val);
            }
        }
        Err(MemoryFault(addr))
    }

    fn write_byte(&self, addr: usize, val: u8) -> Result<(), Fault> {
        let devices = self.devices.read().unwrap();

        for (range, device) in devices.iter() {
            if range.contains(&addr) {
                return device.write_byte(addr - range.start, val);
            }
        }
        Err(MemoryFault(addr))
    }

    fn read_double(&self, addr: usize) -> Result<u64, Fault> {
        let devices = self.devices.read().unwrap();

        for (range, device) in devices.iter() {
            if range.contains(&addr) {
                return device.read_double(addr - range.start);
            }
        }
        Err(MemoryFault(addr))
    }
    fn read_word(&self, addr: usize) -> Result<u32, Fault> {
        let devices = self.devices.read().unwrap();

        for (range, device) in devices.iter() {
            if range.contains(&addr) {
                return device.read_word(addr - range.start);
            }
        }
        Err(MemoryFault(addr))
    }

    fn read_half(&self, addr: usize) -> Result<u16, Fault> {
        let devices = self.devices.read().unwrap();

        for (range, device) in devices.iter() {
            if range.contains(&addr) {
                return device.read_half(addr - range.start);
            }
        }
        Err(MemoryFault(addr))
    }

    fn read_byte(&self, addr: usize) -> Result<u8, Fault> {
        let devices = self.devices.read().unwrap();

        for (range, device) in devices.iter() {
            if range.contains(&addr) {
                return device.read_byte(addr - range.start);
            }
        }
        Err(MemoryFault(addr))
    }
}

#[cfg(test)]
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
