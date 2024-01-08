use std::borrow::Cow;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use gdb_remote_protocol::Signal::{SIGSTOP, SIGTRAP};
use gdb_remote_protocol::{
    Breakpoint, Error, Handler, MemoryRegion, ProcessType, StopReason, ThreadId, VCont, VContFeature,
};
use log::debug;

use crate::device::Device;
use crate::hart::Hart;
use crate::irq::Interrupt;

pub struct Emulator {
    hart: RefCell<Hart>,
    breakpoints: RefCell<Vec<usize>>,
    trap: Arc<AtomicBool>,
}

impl Emulator {
    pub fn new(hart: Hart) -> Emulator {
        Emulator {
            hart: hart.into(),
            breakpoints: RefCell::new(vec![]),
            trap: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Handler for Emulator {
    fn attached(&self, _pid: Option<u64>) -> Result<ProcessType, Error> {
        debug!("process attached");

        signal_hook::flag::register(signal_hook::consts::SIGTRAP, Arc::clone(&self.trap)).unwrap();

        Ok(ProcessType::Attached)
    }

    fn detach(&self, _pid: Option<u64>) -> Result<(), Error> {
        debug!("process detached");
        Ok(())
    }

    fn read_memory(&self, region: MemoryRegion) -> Result<Vec<u8>, Error> {
        let mut result: Vec<u8> = vec![];
        for i in 0..region.length {
            result.push(self.hart.borrow().bus.read_byte((region.address + i) as usize)?);
        }
        Ok(result)
    }

    fn read_general_registers(&self) -> Result<Vec<u8>, Error> {
        debug!("reading registers");
        let mut result = Vec::new();
        for i in 0..32 {
            let reg = self.hart.borrow().get_register(i);
            result.extend_from_slice(&reg.to_le_bytes());
        }
        let reg = self.hart.borrow().get_pc();
        result.extend_from_slice(&reg.to_le_bytes());
        Ok(result)
    }

    fn halt_reason(&self) -> Result<StopReason, Error> {
        debug!("halted");
        Ok(StopReason::Signal(SIGTRAP as u8))
    }

    fn set_address_randomization(&self, _enable: bool) -> Result<(), Error> {
        Ok(())
    }

    fn insert_software_breakpoint(&self, breakpoint: Breakpoint) -> Result<(), Error> {
        let addr = breakpoint.addr as usize;
        if !self.breakpoints.borrow_mut().contains(&addr) {
            self.breakpoints.borrow_mut().push(addr);
        }
        Ok(())
    }

    fn insert_hardware_breakpoint(&self, breakpoint: Breakpoint) -> Result<(), Error> {
        self.insert_software_breakpoint(breakpoint)
    }

    fn remove_software_breakpoint(&self, breakpoint: Breakpoint) -> Result<(), Error> {
        self.breakpoints
            .borrow_mut()
            .retain(|addr| *addr != (breakpoint.addr as usize));
        Ok(())
    }

    fn remove_hardware_breakpoint(&self, breakpoint: Breakpoint) -> Result<(), Error> {
        self.remove_software_breakpoint(breakpoint)
    }

    fn query_supported_vcont(&self) -> Result<Cow<'static, [VContFeature]>, Error> {
        Ok(Cow::from(
            &[
                VContFeature::Continue,
                VContFeature::ContinueWithSignal,
                VContFeature::Step,
                VContFeature::StepWithSignal,
                VContFeature::Stop,
                VContFeature::RangeStep,
            ][..],
        ))
    }

    fn vcont(&self, request: Vec<(VCont, Option<ThreadId>)>) -> Result<StopReason, Error> {
        debug!("continuing");
        let req = request.first().unwrap();
        match &req.0 {
            VCont::Continue => {
                let mut cpu_ref = self.hart.borrow_mut();
                cpu_ref.tick()?;
                while !self.breakpoints.borrow().contains(&cpu_ref.get_pc()) {
                    if self.trap.load(Ordering::Relaxed) {
                        self.trap.store(false, Ordering::Relaxed);
                        return Ok(StopReason::Signal(SIGTRAP as u8));
                    }

                    match cpu_ref.tick() {
                        Ok(_) => continue,
                        Err(e) => {
                            return match e {
                                Interrupt::MemoryFault(_) => Ok(StopReason::Signal(SIGTRAP as u8)),
                                Interrupt::Unmapped(_) => Ok(StopReason::Signal(SIGTRAP as u8)),
                                Interrupt::Unaligned(_) => Err(Error::from(e)),
                                Interrupt::Halt => Err(Error::from(e)),
                                Interrupt::Unimplemented(_) => Ok(StopReason::Signal(SIGTRAP as u8)),
                                Interrupt::InstructionDecodingError => Ok(StopReason::Signal(SIGTRAP as u8)),
                                Interrupt::IllegalOpcode(_) => Ok(StopReason::Signal(SIGTRAP as u8)),
                            }
                        }
                    }
                }
                Ok(StopReason::Signal(SIGTRAP as u8))
            }
            VCont::ContinueWithSignal(sig) => {
                let mut cpu_ref = self.hart.borrow_mut();
                cpu_ref.tick()?;
                while !self.breakpoints.borrow().contains(&cpu_ref.get_pc()) {
                    if self.trap.load(Ordering::Relaxed) {
                        self.trap.store(false, Ordering::Relaxed);
                        return Ok(StopReason::Signal(SIGTRAP as u8));
                    }

                    cpu_ref.tick()?;
                }
                Ok(StopReason::Signal(*sig))
            }
            VCont::RangeStep(range) => {
                let mut cpu_ref = self.hart.borrow_mut();
                cpu_ref.tick()?;
                while !self.breakpoints.borrow().contains(&cpu_ref.get_pc())
                    && range.contains(&(cpu_ref.get_pc() as u64))
                {
                    if self.trap.load(Ordering::Relaxed) {
                        self.trap.store(false, Ordering::Relaxed);
                        return Ok(StopReason::Signal(SIGTRAP as u8));
                    }

                    cpu_ref.tick()?;
                }
                Ok(StopReason::Signal(SIGTRAP as u8))
            }
            VCont::Step => {
                self.hart.borrow_mut().tick()?;
                Ok(StopReason::Signal(SIGTRAP as u8))
            }
            VCont::StepWithSignal(sig) => {
                self.hart.borrow_mut().tick()?;
                Ok(StopReason::Signal(*sig))
            }
            VCont::Stop => Ok(StopReason::Signal(SIGSTOP as u8)),
        }
    }
}

impl From<Interrupt> for gdb_remote_protocol::Error {
    fn from(value: Interrupt) -> Self {
        match value {
            Interrupt::MemoryFault(_) => Error::Error(0),
            Interrupt::Unmapped(_) => Error::Error(1),
            Interrupt::Unaligned(_) => Error::Error(2),
            Interrupt::Halt => Error::Error(3),
            Interrupt::Unimplemented(_) => Error::Unimplemented,
            Interrupt::InstructionDecodingError => Error::Error(4),
            Interrupt::IllegalOpcode(_) => Error::Error(5),
        }
    }
}
