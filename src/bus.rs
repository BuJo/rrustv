use crate::device::Device;
use crate::plic::Fault;
use crate::plic::Fault::MemoryFault;
use crate::ram::Ram;
use crate::rom::Rom;

pub static RAM_ADDR: usize = 0x80000000;

pub struct Bus {
    rom: Rom,
    ram: Ram,
}

impl Bus {
    pub fn new(rom: Rom, ram: Ram) -> Bus {
        Self { rom, ram }
    }
}

impl Device for Bus {
    fn write_word(&self, addr: usize, val: u32) -> Result<(), Fault> {
        match addr {
            0x80000000.. => self.ram.write_word(addr - RAM_ADDR, val),
            _ => Err(MemoryFault(addr)),
        }
    }

    fn write_half(&self, addr: usize, val: u16) -> Result<(), Fault> {
        match addr {
            0x80000000.. => self.ram.write_half(addr - RAM_ADDR, val),
            _ => Err(MemoryFault(addr)),
        }
    }

    fn write_byte(&self, addr: usize, val: u8) -> Result<(), Fault> {
        match addr {
            0x80000000.. => self.ram.write_byte(addr - RAM_ADDR, val),
            _ => Err(MemoryFault(addr)),
        }
    }

    fn read_word(&self, addr: usize) -> Result<u32, Fault> {
        match addr {
            0x0000..=0x1FFF => self.rom.read_word(addr),
            0x80000000.. => self.ram.read_word(addr - RAM_ADDR),
            _ => Err(MemoryFault(addr)),
        }
    }

    fn read_half(&self, addr: usize) -> Result<u16, Fault> {
        match addr {
            0x0000..=0x1FFF => self.rom.read_half(addr),
            0x80000000.. => self.ram.read_half(addr - RAM_ADDR),
            _ => Err(MemoryFault(addr)),
        }
    }

    fn read_byte(&self, addr: usize) -> Result<u8, Fault> {
        match addr {
            0x0000..=0x1FFF => self.rom.read_byte(addr),
            0x80000000.. => self.ram.read_byte(addr - RAM_ADDR),
            _ => Err(MemoryFault(addr)),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::bus::Bus;
    use crate::device::Device;
    use crate::ram::Ram;
    use crate::rom::Rom;

    fn bus() -> Bus {
        let rom = Rom::new(vec![1, 2, 3, 4]);
        let ram = Ram::new();

        Bus::new(rom, ram)
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
        let can_write = bus.write_word(0x80000000, 0x1).is_ok();

        assert_eq!(can_write, true, "ram should be writeable");
    }
}
