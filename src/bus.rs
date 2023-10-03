use crate::bus::Fault::MemoryFault;
use crate::ram::Ram;
use crate::rom::Rom;
use crate::rtc::Rtc;

pub struct Bus {
    rom: Rom,
    ram: Ram,
    rtc: Rtc,
}

#[derive(Debug)]
pub enum Fault {
    MemoryFault,
}

impl Bus {
    pub fn new(rom: Rom, ram: Ram, rtc: Rtc) -> Bus {
        Self { rom, ram, rtc }
    }

    pub fn write_word(&self, addr: usize, val: u32) -> Result<(), Fault> {
        match addr {
            0x4000..=0x4FFF => self.rtc.write_word(addr, val),
            0x8000.. => Ok(self.ram.write_word(addr - 0x8000, val)),
            _ => Err(MemoryFault),
        }
    }

    pub fn write_byte(&self, addr: usize, val: u8) -> Result<(), Fault> {
        match addr {
            0x8000.. => Ok(self.ram.write_byte(addr - 0x8000, val)),
            _ => Err(MemoryFault),
        }
    }

    pub fn read_word(&self, addr: usize) -> Result<u32, Fault> {
        match addr {
            0x0000..=0x1FFF => self
                .rom
                .read_word(addr)
                .or_else(|| Some(0))
                .ok_or(MemoryFault),
            0x4000..=0x4FFF => self.rtc.read_word(addr),
            0x8000.. => self.ram.read_word(addr - 0x8000).ok_or(MemoryFault),
            _ => Err(MemoryFault),
        }
    }

    pub fn read_byte(&self, addr: usize) -> Result<u8, Fault> {
        match addr {
            0x0000..=0x1FFF => self
                .rom
                .read_byte(addr)
                .or_else(|| Some(0))
                .ok_or(MemoryFault),
            0x8000.. => self.ram.read_byte(addr - 0x8000).ok_or(MemoryFault),
            _ => Err(MemoryFault),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::ram::Ram;
    use crate::rom::Rom;
    use crate::rtc::Rtc;

    fn bus() -> Bus {
        let rom = Rom::new(vec![1, 2, 3, 4]);
        let ram = Ram::new();
        let rtc = Rtc::new();

        Bus::new(rom, ram, rtc)
    }

    #[test]
    fn non_writeable_rom() {
        let bus = bus();
        let can_write = bus.write_word(0, 0x0).is_ok();

        assert_eq!(can_write, false, "rom should not be writeable");
    }

    #[test]
    fn writeable_ram() {
        let bus = bus();
        let can_write = bus.write_word(0x8000, 0x1).is_ok();

        assert_eq!(can_write, true, "ram should be writeable");
    }
}
