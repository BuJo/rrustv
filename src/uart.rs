use crate::device::Device;
use crate::irq::Interrupt;
use log::trace;
use std::io;
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};

pub struct Uart8250 {
    ie: AtomicBool,
}

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
        Uart8250 {
            ie: AtomicBool::new(true),
        }
    }
}

impl Default for Uart8250 {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for Uart8250 {
    fn write_double(&self, _addr: usize, _val: u64) -> Result<(), Interrupt> {
        Err(Interrupt::Unimplemented("8250: writing double unimplemented".into()))
    }

    fn write_word(&self, _addr: usize, _val: u32) -> Result<(), Interrupt> {
        Err(Interrupt::Unimplemented("8250: writing word unimplemented".into()))
    }

    fn write_half(&self, _addr: usize, _val: u16) -> Result<(), Interrupt> {
        Err(Interrupt::Unimplemented("8250: writing halfword unimplemented".into()))
    }

    fn write_byte(&self, addr: usize, val: u8) -> Result<(), Interrupt> {
        // Emulating a 8250 / 16550 UART
        match addr {
            Uart8250::RX => {
                print!("{}", val as char);

                // only flush on special chars
                if !(0x20..0x7e).contains(&val) {
                    io::stdout().flush().unwrap();
                }
                Ok(())
            }
            Uart8250::IER => {
                if val == 0 {
                    trace!("8250: disabling interrupts");
                    self.ie.store(false, Ordering::Relaxed);
                } else {
                    trace!("8250: enabling interrupts");
                    self.ie.store(true, Ordering::Relaxed);
                }
                Ok(())
            }
            Uart8250::FCR => {
                trace!("8250: FIFO control: {:0b}", val);
                Ok(())
            }
            Uart8250::LCR => {
                trace!("8250: Line control: {:0b}", val);
                Ok(())
            }
            Uart8250::MCR => {
                trace!("8250: Modem control: {:0b}", val);
                Ok(())
            }
            _ => Err(Interrupt::Unimplemented(format!(
                "8250: writing to unknown byte address 0x{:x}: {}",
                addr, val
            ))),
        }
    }

    fn read_double(&self, _addr: usize) -> Result<u64, Interrupt> {
        Err(Interrupt::Unimplemented("8250: reading double unimplemented".into()))
    }

    fn read_word(&self, _addr: usize) -> Result<u32, Interrupt> {
        Err(Interrupt::Unimplemented("8250: reading word unimplemented".into()))
    }

    fn read_half(&self, _addr: usize) -> Result<u16, Interrupt> {
        Err(Interrupt::Unimplemented("8250: reading halfword unimplemented".into()))
    }

    fn read_byte(&self, addr: usize) -> Result<u8, Interrupt> {
        // Emulating a 8250 / 16550 UART
        let have_data: bool = false; // XXX: need a way to detect presence of data in stdin

        match addr {
            Uart8250::LSR if have_data => {
                let mut buffer = [0];
                io::stdin().read_exact(&mut buffer)?;
                Ok(buffer[0])
            }
            Uart8250::IER => Ok(self.ie.load(Ordering::Relaxed) as u8),
            Uart8250::LSR => Ok(0x60 | have_data as u8),
            Uart8250::LCR => Ok(0b11),
            _ => Err(Interrupt::Unimplemented(format!("8250: reading addr {}", addr))),
        }
    }
}

impl From<io::Error> for Interrupt {
    fn from(_value: io::Error) -> Self {
        Interrupt::Unimplemented("8250: io error not handled".into())
    }
}
