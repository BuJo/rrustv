use crate::device::Device;
use crate::irq::Interrupt;
use log::trace;
use std::collections::HashMap;
use std::sync::RwLock;

pub struct Plic {
    interrupt_bits: RwLock<HashMap<usize, u32>>,
}

impl Plic {
    pub fn new() -> Plic {
        Plic {
            interrupt_bits: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for Plic {
    fn default() -> Self {
        Self::new()
    }
}

impl Device for Plic {
    fn write_double(&self, addr: usize, val: u64) -> Result<(), Interrupt> {
        trace!("writing double word to 0x{:x} = {}", addr, val);
        Ok(())
    }

    fn write_word(&self, addr: usize, val: u32) -> Result<(), Interrupt> {
        match addr {
            0x0..=0x000FFC => {} //trace!("setting interrupt priority: {} -> {}", addr, val),
            0x001000..=0x00107C => {
                trace!("setting interrupt bits pending: {} -> {:b}", addr, val)
            }
            0x002000..=0x1F1FFC => {
                // setting interrupt bits to enabled
                // trace!(
                //     "setting interrupt bits enabled: {} -> {:b}",
                //     addr,
                //     val
                // );
                let mut bits = self.interrupt_bits.write().unwrap();
                bits.insert(addr, val);
            }
            0x200000..=0x3FFF000 if addr & 0b111 == 0 => {
                let _ctx = (addr / 4096) - (0x200000 / 4096);
                // trace!(
                //     "setting priority threshold for context: {} -> {}",
                //     ctx,
                //     0
                // );
            }
            0x200000..=0x3FFF000 if addr & 0b111 == 0x4 => {
                let ctx = ((addr - 0x4) / 4096) - (0x200000 / 4096);
                trace!("completing interrupt for context: {} -> {}", ctx, 0);
            }
            _ => trace!("writing word to 0x{:x} = {}", addr, val),
        }

        Ok(())
    }

    fn write_half(&self, _addr: usize, _val: u16) -> Result<(), Interrupt> {
        Err(Interrupt::Unimplemented(
            "writing half word unimplemented".into(),
        ))
    }

    fn write_byte(&self, _addr: usize, _val: u8) -> Result<(), Interrupt> {
        Err(Interrupt::Unimplemented(
            "writing byte unimplemented".into(),
        ))
    }

    fn read_double(&self, addr: usize) -> Result<u64, Interrupt> {
        Ok(self.read_word(addr)? as u64 | (self.read_word(addr + 4)? as u64) << 32)
    }

    fn read_word(&self, addr: usize) -> Result<u32, Interrupt> {
        match addr {
            0x000000..=0x000FFC => {
                trace!("reading interrupt source priority: {} -> {:b}", addr / 4, 0)
            }
            0x002000..=0x1F1FFC => {
                // checking if interrupt bits are enabled
                let bits = self.interrupt_bits.read().unwrap();
                let bits = bits.get(&addr).copied().unwrap_or(0);

                //trace!("reading interrupt bits enabled: {} -> {:b}", addr, bits);
                return Ok(bits);
            }
            0x200000..=0x3FFF000 if addr & 0b111 == 0 => {
                let ctx = (addr / 4096) - (0x200000 / 4096);
                trace!("reading priority threshold for context: {} -> {}", ctx, 0);
            }
            0x200000..=0x3FFF000 if addr & 0b111 == 0x4 => {
                let ctx = ((addr - 0x4) / 4096) - (0x200000 / 4096);
                trace!("claiming interrupt for context: {} -> {}", ctx, 0);
            }
            _ => trace!("reading word from 0x{:x}", addr),
        }
        Ok(0)
    }

    fn read_half(&self, _addr: usize) -> Result<u16, Interrupt> {
        Err(Interrupt::Unimplemented(
            "reading half word unimplemented".into(),
        ))
    }

    fn read_byte(&self, _addr: usize) -> Result<u8, Interrupt> {
        Err(Interrupt::Unimplemented(
            "reading byte unimplemented".into(),
        ))
    }
}
