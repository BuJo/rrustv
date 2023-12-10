use crate::device::Device;
use crate::plic::Fault;
use std::io;
use std::io::{Read, Write};

pub struct Uart8250 {}

#[allow(unused)]
impl Uart8250 {
    const RX: usize = 0; // In: Transmit buffer
    const IER: usize = 1; // In: Interrupt Enable Register
    const LCR: usize = 3; // In/Out: Line Control Register
    const MCR: usize = 4; // In: Modem Control Register
    const LSR: usize = 5; // Out:  Line Status Register
    const FCR: usize = 2; // In: FIFO Control Register
    const DLL: usize = 0; // In: Divisor Latch Low
    const DLM: usize = 1; // In: Divisor Latch Low

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
        match addr {
            Uart8250::RX => {
                print!("{}", val as char);

                // only flush on special chars
                if !(0x20..0x7e).contains(&val) {
                    io::stdout().flush().unwrap();
                }
            }
            _ => {}
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

        match addr {
            Uart8250::LSR if have_data => {
                let mut buffer = [0];
                io::stdin().read_exact(&mut buffer)?;
                Ok(buffer[0])
            }
            Uart8250::LSR => Ok(0x60 | have_data as u8),
            Uart8250::LCR => Ok(0b0_0_000_0_11),
            _ => Ok(0),
        }
    }
}

impl From<io::Error> for Fault {
    fn from(_value: io::Error) -> Self {
        Fault::MemoryFault(0)
    }
}
