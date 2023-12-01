use std::io;
use std::io::{Read, Write};
use crate::device::Device;
use crate::plic::Fault;

pub struct Uart8250 {}

impl Uart8250 {
    pub fn new() -> Uart8250 {
        Uart8250 {}
    }
}

impl Device for Uart8250 {
    fn write_double(&self, _addr: usize, _val: u64) -> Result<(), Fault> {
        Err(Fault::Unimplemented)
    }

    fn write_word(&self, _addr: usize, _val: u32) -> Result<(), Fault> {
        Err(Fault::Unimplemented)
    }

    fn write_half(&self, _addr: usize, _val: u16) -> Result<(), Fault> {
        Err(Fault::Unimplemented)
    }

    fn write_byte(&self, addr: usize, val: u8) -> Result<(), Fault> {
        // Emulating a 8250 / 16550 UART
        if addr == 0x10000000 {
            print!("{}", val);
            io::stdout().flush().unwrap();
        }
        Ok(())
    }

    fn read_double(&self, _addr: usize) -> Result<u64, Fault> {
        Err(Fault::Unimplemented)
    }

    fn read_word(&self, _addr: usize) -> Result<u32, Fault> {
        Err(Fault::Unimplemented)
    }

    fn read_half(&self, _addr: usize) -> Result<u16, Fault> {
        Err(Fault::Unimplemented)
    }

    fn read_byte(&self, addr: usize) -> Result<u8, Fault> {
        // Emulating a 8250 / 16550 UART
        let have_data: bool = false; // XXX: need a way to detect presence of data in stdin
        if addr == 0x10000005 {
            Ok(0x60 | have_data as u8)
        } else if addr == 0x10000000 && have_data {
            let mut buffer = [0];
            io::stdin().read_exact(&mut buffer)?;
            Ok(buffer[0])
        } else {
            Ok(0)
        }
    }
}

impl From<io::Error> for Fault {
    fn from(_value: io::Error) -> Self {
        Fault::MemoryFault(0)
    }
}
