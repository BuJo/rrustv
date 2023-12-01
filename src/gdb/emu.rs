use std::borrow::Cow;
use std::cell::RefCell;

use gdb_remote_protocol::Signal::SIGTRAP;
use gdb_remote_protocol::{
    Breakpoint, Error, Handler, Id, MemoryRegion, ProcessType, StopReason, ThreadId, VCont,
    VContFeature,
};
use log::debug;

use crate::device::Device;
use crate::dynbus::DynBus;
use crate::hart::Hart;
use crate::plic::Fault;

pub struct Emulator {
    hart: RefCell<Hart<DynBus>>,
    hart_id: ThreadId,
    breakpoints: RefCell<Vec<usize>>,
}

impl Emulator {
    pub fn new(hart: Hart<DynBus>) -> Emulator {
        Emulator {
            hart: hart.into(),
            hart_id: ThreadId {
                pid: Id::Id(1),
                tid: Id::Id(1),
            },
            breakpoints: RefCell::new(vec![]),
        }
    }
}

impl Handler for Emulator {
    fn attached(&self, _pid: Option<u64>) -> Result<ProcessType, Error> {
        debug!("process attached");
        Ok(ProcessType::Attached)
    }

    fn detach(&self, _pid: Option<u64>) -> Result<(), Error> {
        debug!("process detached");
        Ok(())
    }

    fn read_memory(&self, region: MemoryRegion) -> Result<Vec<u8>, Error> {
        let mut result: Vec<u8> = vec![];
        for i in 0..region.length {
            result.push(
                self.hart
                    .borrow()
                    .bus
                    .read_byte((region.address + i) as usize)?,
            );
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
            ][..],
        ))
    }

    fn vcont(&self, request: Vec<(VCont, Option<ThreadId>)>) -> Result<StopReason, Error> {
        debug!("continuing");
        let req = request.first().unwrap();
        match req.0 {
            VCont::Continue => {
                let mut cpu_ref = self.hart.borrow_mut();
                cpu_ref.tick()?;
                while !self.breakpoints.borrow().contains(&cpu_ref.get_pc()) {
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
                Ok(StopReason::Signal(sig))
            }
            _ => Err(Error::Unimplemented),
        }
    }
}

impl From<Fault> for gdb_remote_protocol::Error {
    fn from(value: Fault) -> Self {
        match value {
            Fault::MemoryFault(_) => Error::Error(0),
            Fault::Unaligned(_) => Error::Error(2),
            Fault::Halt => Error::Error(3),
            Fault::Unimplemented => Error::Unimplemented,
            Fault::InstructionDecodingError => Error::Error(4),
            Fault::IllegalOpcode(_) => Error::Error(5),
        }
    }
}
