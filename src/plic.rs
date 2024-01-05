use std::collections::HashMap;
use std::sync::Mutex;

use log::{debug, trace, warn};

use crate::device::Device;
use crate::irq::Interrupt;

struct Context {
    claimed: bool,
    threshold: u32,
    enabled: HashMap<usize, u32>,
}

impl Default for Context {
    fn default() -> Self {
        Context {
            claimed: false,
            threshold: 0,
            enabled: HashMap::new(),
        }
    }
}

struct Source {
    priority: u32,
    interrupts: u32,
}

impl Default for Source {
    fn default() -> Self {
        Source {
            priority: 0,
            interrupts: 0,
        }
    }
}

pub struct Plic {
    sources: Mutex<HashMap<usize, Source>>,
    contexts: Mutex<HashMap<usize, Context>>,
}

impl Plic {
    pub fn new() -> Plic {
        Plic {
            sources: Mutex::new(HashMap::new()),
            contexts: Mutex::new(HashMap::new()),
        }
    }

    fn fire_interrupt(&self, source: usize, bits: u32) {
        debug!("source {}: setting interrupt bits pending: {:0b}", source, bits);
        let mut sources = self.sources.lock().unwrap();
        let source = sources.entry(source).or_default();
        source.interrupts = bits;
    }

    fn set_source_priority(&self, source: usize, priority: u32) {
        trace!("source {}: setting interrupt priority: {}", source, priority);
        let mut sources = self.sources.lock().unwrap();
        let source = sources.entry(source).or_default();
        source.priority = priority;
    }

    fn get_source_priority(&self, source: usize) -> u32 {
        let mut sources = self.sources.lock().unwrap();
        let src = sources.entry(source).or_default();

        trace!(
            "source {}: reading priority threshold for context: {}",
            source,
            src.priority
        );

        src.priority
    }

    fn claim_interrupt(&self, context: usize) {
        debug!("context {}: claiming interrupt", context);
        let mut contexts = self.contexts.lock().unwrap();
        let context = contexts.entry(context).or_default();
        context.claimed = true;
    }

    fn complete_interrupt(&self, context: usize, id: u32) {
        debug!("context {}: completing interrupt {}", context, id);
        let mut contexts = self.contexts.lock().unwrap();
        let context = contexts.entry(context).or_default();
        context.claimed = false;
    }

    fn set_source_enabled(&self, context: usize, bit_offset: usize, source_bits: u32) {
        trace!(
            "context {}: enabling sources: {:032b}[{}]",
            context,
            source_bits,
            bit_offset
        );
        let x = bit_offset / 4 / 4;
        let mut contexts = self.contexts.lock().unwrap();
        let context = contexts.entry(context).or_default();
        let enabled_sources = context.enabled.entry(x).or_default();
        *enabled_sources = source_bits;
    }

    fn get_source_enabled(&self, context: usize, bit_offset: usize) -> u32 {
        let x = bit_offset / 4 / 4;
        let mut contexts = self.contexts.lock().unwrap();
        let ctx = contexts.entry(context).or_default();
        let enabled = *ctx.enabled.entry(x).or_default();

        trace!(
            "context {}: sources enabled enabled: {:032b}[{}]",
            context,
            enabled,
            bit_offset
        );

        enabled
    }

    fn set_priority_threshold(&self, context: usize, threshold: u32) {
        trace!("context {}: setting priority threshold to {}", context, threshold);
        let mut contexts = self.contexts.lock().unwrap();
        let context = contexts.entry(context).or_default();
        context.threshold = threshold;
    }

    fn get_priority_threshold(&self, context: usize) -> u32 {
        trace!("context {}: getting priority threshold: {}", context, 0);
        let mut contexts = self.contexts.lock().unwrap();
        contexts.entry(context).or_default().threshold
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
            0x0..=0x000FFC => {
                let source = addr / 4;
                self.set_source_priority(source, val)
            }
            0x001000..=0x00107C => {
                let base = addr - 0x001000;
                let source = base / 4;
                self.fire_interrupt(source, val);
            }
            0x002000..=0x1F1FFC => {
                let base = addr - 0x002000;
                let ctx = base / 0x80;
                let bit_offset = ((base % 0x80) / 4) * 32;
                self.set_source_enabled(ctx, bit_offset, val)
            }
            0x200000..=0x3FFF000 if addr & 0b111 == 0 => {
                let base = addr - 0x200000;
                let ctx = base / 0x1000;
                self.set_priority_threshold(ctx, val)
            }
            0x200000..=0x3FFF000 if addr & 0b111 == 0x4 => {
                let base = addr - 0x200004;
                let ctx = base / 0x1000;
                self.complete_interrupt(ctx, val);
            }
            _ => warn!("writing word to 0x{:x} = {}", addr, val),
        }

        Ok(())
    }

    fn write_half(&self, _addr: usize, _val: u16) -> Result<(), Interrupt> {
        Err(Interrupt::Unimplemented("writing half word unimplemented".into()))
    }

    fn write_byte(&self, _addr: usize, _val: u8) -> Result<(), Interrupt> {
        Err(Interrupt::Unimplemented("writing byte unimplemented".into()))
    }

    fn read_double(&self, addr: usize) -> Result<u64, Interrupt> {
        Ok(self.read_word(addr)? as u64 | (self.read_word(addr + 4)? as u64) << 32)
    }

    fn read_word(&self, addr: usize) -> Result<u32, Interrupt> {
        match addr {
            0x000000..=0x000FFC => {
                let source = addr / 4;
                Ok(self.get_source_priority(source))
            }
            0x002000..=0x1F1FFC => {
                let base = addr - 0x002000;
                let ctx = base / 0x80;
                let bit_offset = ((base % 0x80) / 4) * 32;
                Ok(self.get_source_enabled(ctx, bit_offset))
            }
            0x200000..=0x3FFF000 if addr & 0b111 == 0 => {
                let base = addr - 0x200000;
                let ctx = base / 0x1000;
                Ok(self.get_priority_threshold(ctx))
            }
            0x200000..=0x3FFF000 if addr & 0b111 == 0x4 => {
                let base = addr - 0x200004;
                let ctx = base / 0x1000;
                self.claim_interrupt(ctx);
                Ok(0)
            }
            _ => {
                warn!("reading word from 0x{:x}", addr);
                Err(Interrupt::Unimplemented("plic".into()))
            }
        }
    }

    fn read_half(&self, _addr: usize) -> Result<u16, Interrupt> {
        Err(Interrupt::Unimplemented("reading half word unimplemented".into()))
    }

    fn read_byte(&self, _addr: usize) -> Result<u8, Interrupt> {
        Err(Interrupt::Unimplemented("reading byte unimplemented".into()))
    }
}
