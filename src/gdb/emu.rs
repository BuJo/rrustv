use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use gdbstub::common::Signal;
use gdbstub::target::ext::base::single_register_access::{SingleRegisterAccess, SingleRegisterAccessOps};
use gdbstub::target::ext::base::singlethread::{
    SingleThreadBase, SingleThreadResume, SingleThreadResumeOps, SingleThreadSingleStep, SingleThreadSingleStepOps,
};
use gdbstub::target::ext::breakpoints::{
    Breakpoints, BreakpointsOps, HwBreakpoint, HwBreakpointOps, SwBreakpoint, SwBreakpointOps,
};
use gdbstub::target::{Target, TargetError, TargetResult};
use gdbstub_arch::riscv::Riscv64;
use gdbstub_arch::riscv::reg::id::RiscvRegId;
use log::debug;

use crate::device::Device;
use crate::hart::Hart;
use crate::irq::Interrupt;

/// Stop reason returned after execution completes
#[derive(Debug, Clone, Copy)]
pub enum ExecMode {
    Continue,
    Step,
    RangeStep { start: u64, end: u64 },
}

/// Number of instructions to execute per batch before checking for GDB input
const INSTRUCTIONS_PER_BATCH: usize = 10000;

/// Result of running the emulator
#[derive(Debug, Clone, Copy)]
pub enum StopReason {
    Halted,
    Signal(Signal),
    Breakpoint,
    DoneStep,
    /// Still running, returned periodically to allow checking for GDB interrupts
    Running,
}

pub struct Emulator {
    pub hart: Hart,
    breakpoints: Vec<u64>,
    trap: Arc<AtomicBool>,
    exec_mode: Option<ExecMode>,
}

impl Emulator {
    pub fn new(hart: Hart) -> Emulator {
        let trap = Arc::new(AtomicBool::new(false));
        signal_hook::flag::register(signal_hook::consts::SIGTRAP, Arc::clone(&trap)).unwrap();

        Emulator {
            hart,
            breakpoints: Vec::new(),
            trap,
            exec_mode: None,
        }
    }

    pub fn set_exec_mode(&mut self, mode: ExecMode) {
        self.exec_mode = Some(mode);
    }

    /// Execute until breakpoint, trap signal, or error
    pub fn run(&mut self) -> StopReason {
        match self.exec_mode.take() {
            Some(ExecMode::Continue) => self.run_continue(),
            Some(ExecMode::Step) => self.run_step(),
            Some(ExecMode::RangeStep { start, end }) => self.run_range_step(start, end),
            None => StopReason::Halted,
        }
    }

    fn run_continue(&mut self) -> StopReason {
        for _ in 0..INSTRUCTIONS_PER_BATCH {
            if self.breakpoints.contains(&(self.hart.get_pc() as u64)) {
                return StopReason::Breakpoint;
            }

            if self.trap.load(Ordering::Relaxed) {
                self.trap.store(false, Ordering::Relaxed);
                return StopReason::Signal(Signal::SIGTRAP);
            }

            match self.hart.tick() {
                Ok(_) => continue,
                Err(e) => return self.handle_interrupt(e),
            }
        }
        // Batch complete, return to check for GDB input
        self.exec_mode = Some(ExecMode::Continue);
        StopReason::Running
    }

    fn run_step(&mut self) -> StopReason {
        match self.hart.tick() {
            Ok(_) => StopReason::DoneStep,
            Err(e) => self.handle_interrupt(e),
        }
    }

    fn run_range_step(&mut self, start: u64, end: u64) -> StopReason {
        for _ in 0..INSTRUCTIONS_PER_BATCH {
            let pc = self.hart.get_pc() as u64;

            // Stop if PC is outside the range
            if pc < start || pc >= end {
                return StopReason::DoneStep;
            }

            if self.breakpoints.contains(&pc) {
                return StopReason::Breakpoint;
            }

            if self.trap.load(Ordering::Relaxed) {
                self.trap.store(false, Ordering::Relaxed);
                return StopReason::Signal(Signal::SIGTRAP);
            }

            match self.hart.tick() {
                Ok(_) => continue,
                Err(e) => return self.handle_interrupt(e),
            }
        }
        // Batch complete, return to check for GDB input
        self.exec_mode = Some(ExecMode::RangeStep { start, end });
        StopReason::Running
    }

    fn handle_interrupt(&self, interrupt: Interrupt) -> StopReason {
        match interrupt {
            Interrupt::Halt => StopReason::Halted,
            Interrupt::MemoryFault(_)
            | Interrupt::Unmapped(_)
            | Interrupt::Unimplemented(_)
            | Interrupt::InstructionDecodingError
            | Interrupt::IllegalOpcode(_) => StopReason::Signal(Signal::SIGTRAP),
            Interrupt::Unaligned(_) => StopReason::Signal(Signal::SIGBUS),
        }
    }
}

/// Custom error type for the emulator
#[derive(Debug)]
pub enum EmulatorError {
    MemoryFault(usize),
    Unmapped(usize),
    Unaligned(usize),
    Halt,
    Unimplemented(String),
    InstructionDecodingError,
    IllegalOpcode,
}

impl From<Interrupt> for EmulatorError {
    fn from(value: Interrupt) -> Self {
        match value {
            Interrupt::MemoryFault(addr) => EmulatorError::MemoryFault(addr),
            Interrupt::Unmapped(addr) => EmulatorError::Unmapped(addr),
            Interrupt::Unaligned(addr) => EmulatorError::Unaligned(addr),
            Interrupt::Halt => EmulatorError::Halt,
            Interrupt::Unimplemented(msg) => EmulatorError::Unimplemented(msg),
            Interrupt::InstructionDecodingError => EmulatorError::InstructionDecodingError,
            Interrupt::IllegalOpcode(_) => EmulatorError::IllegalOpcode,
        }
    }
}

impl Target for Emulator {
    type Arch = Riscv64;
    type Error = EmulatorError;

    fn base_ops(&mut self) -> gdbstub::target::ext::base::BaseOps<'_, Self::Arch, Self::Error> {
        gdbstub::target::ext::base::BaseOps::SingleThread(self)
    }

    fn support_breakpoints(&mut self) -> Option<BreakpointsOps<'_, Self>> {
        Some(self)
    }
}

impl SingleThreadBase for Emulator {
    fn read_registers(&mut self, regs: &mut gdbstub_arch::riscv::reg::RiscvCoreRegs<u64>) -> TargetResult<(), Self> {
        debug!("reading registers");
        for i in 0..32 {
            regs.x[i] = self.hart.get_register(i as u8);
        }
        regs.pc = self.hart.get_pc() as u64;
        Ok(())
    }

    fn write_registers(&mut self, regs: &gdbstub_arch::riscv::reg::RiscvCoreRegs<u64>) -> TargetResult<(), Self> {
        debug!("writing registers");
        for i in 0..32 {
            self.hart.set_register(i as u8, regs.x[i]);
        }
        self.hart.set_pc(regs.pc as usize);
        Ok(())
    }

    fn read_addrs(&mut self, start_addr: u64, data: &mut [u8]) -> TargetResult<usize, Self> {
        for (i, byte) in data.iter_mut().enumerate() {
            match self.hart.bus.read_byte((start_addr as usize) + i) {
                Ok(b) => *byte = b,
                Err(_) => return Ok(i), // Return number of bytes successfully read
            }
        }
        Ok(data.len())
    }

    fn write_addrs(&mut self, start_addr: u64, data: &[u8]) -> TargetResult<(), Self> {
        for (i, byte) in data.iter().enumerate() {
            if let Err(e) = self.hart.bus.write_byte((start_addr as usize) + i, *byte) {
                return Err(TargetError::Fatal(e.into()));
            }
        }
        Ok(())
    }

    fn support_resume(&mut self) -> Option<SingleThreadResumeOps<'_, Self>> {
        Some(self)
    }

    fn support_single_register_access(&mut self) -> Option<SingleRegisterAccessOps<'_, (), Self>> {
        Some(self)
    }
}

impl SingleRegisterAccess<()> for Emulator {
    fn read_register(&mut self, _tid: (), reg_id: RiscvRegId<u64>, buf: &mut [u8]) -> TargetResult<usize, Self> {
        let value = match reg_id {
            RiscvRegId::Gpr(n) => self.hart.get_register(n),
            RiscvRegId::Pc => self.hart.get_pc() as u64,
            _ => return Ok(0), // Unsupported register
        };
        let bytes = value.to_le_bytes();
        let len = buf.len().min(bytes.len());
        buf[..len].copy_from_slice(&bytes[..len]);
        Ok(len)
    }

    fn write_register(&mut self, _tid: (), reg_id: RiscvRegId<u64>, val: &[u8]) -> TargetResult<(), Self> {
        let mut bytes = [0u8; 8];
        let len = val.len().min(8);
        bytes[..len].copy_from_slice(&val[..len]);
        let value = u64::from_le_bytes(bytes);

        match reg_id {
            RiscvRegId::Gpr(n) => self.hart.set_register(n, value),
            RiscvRegId::Pc => self.hart.set_pc(value as usize),
            _ => return Ok(()), // Ignore unsupported registers
        }
        Ok(())
    }
}

impl SingleThreadResume for Emulator {
    fn resume(&mut self, signal: Option<Signal>) -> Result<(), Self::Error> {
        debug!("resuming with signal: {:?}", signal);
        self.exec_mode = Some(ExecMode::Continue);
        Ok(())
    }

    fn support_single_step(&mut self) -> Option<SingleThreadSingleStepOps<'_, Self>> {
        Some(self)
    }
}

impl SingleThreadSingleStep for Emulator {
    fn step(&mut self, signal: Option<Signal>) -> Result<(), Self::Error> {
        debug!("stepping with signal: {:?}", signal);
        self.exec_mode = Some(ExecMode::Step);
        Ok(())
    }
}

impl Breakpoints for Emulator {
    fn support_sw_breakpoint(&mut self) -> Option<SwBreakpointOps<'_, Self>> {
        Some(self)
    }

    fn support_hw_breakpoint(&mut self) -> Option<HwBreakpointOps<'_, Self>> {
        Some(self)
    }
}

impl SwBreakpoint for Emulator {
    fn add_sw_breakpoint(&mut self, addr: u64, _kind: usize) -> TargetResult<bool, Self> {
        debug!("adding software breakpoint at {:#x}", addr);
        if !self.breakpoints.contains(&addr) {
            self.breakpoints.push(addr);
        }
        Ok(true)
    }

    fn remove_sw_breakpoint(&mut self, addr: u64, _kind: usize) -> TargetResult<bool, Self> {
        debug!("removing software breakpoint at {:#x}", addr);
        self.breakpoints.retain(|&a| a != addr);
        Ok(true)
    }
}

impl HwBreakpoint for Emulator {
    fn add_hw_breakpoint(&mut self, addr: u64, _kind: usize) -> TargetResult<bool, Self> {
        debug!("adding hardware breakpoint at {:#x}", addr);
        // Treat hardware breakpoints same as software for this emulator
        if !self.breakpoints.contains(&addr) {
            self.breakpoints.push(addr);
        }
        Ok(true)
    }

    fn remove_hw_breakpoint(&mut self, addr: u64, _kind: usize) -> TargetResult<bool, Self> {
        debug!("removing hardware breakpoint at {:#x}", addr);
        self.breakpoints.retain(|&a| a != addr);
        Ok(true)
    }
}
