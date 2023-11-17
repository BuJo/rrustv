use crate::dynbus::DynBus;
use crate::hart::Hart;
use crate::plic::Fault;
use crate::ram::Ram;
use crate::reg::treg;
use gdbstub::arch::Arch;
use gdbstub::common::Tid;
use gdbstub::target;
use gdbstub::target::ext::base::BaseOps;
use gdbstub::target::ext::base::BaseOps::MultiThread;
use gdbstub::target::TargetResult;
use object::{Object, ObjectSection};
use std::ops::Range;
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::sync::{mpsc, Arc};
use std::thread;

enum EmulationCommand {
    AddBreakpoint(usize),
    RemoveBreakpoint(usize),
    ReadRegisters(Sender<Vec<u64>>),
    SetRegisters(Vec<u64>),
    ReadMemory(Sender<Vec<u8>>, usize, usize),
    WriteMemory(usize, Vec<u8>),
    Resume,
    SetResumeAction(ExecutionMode),
    ClearResumeAction,
}

enum ExecutionMode {
    Continue,
    Halt,
    Step,
}

pub struct Emulator {
    bus: Arc<DynBus>,
    sender: Sender<EmulationCommand>,
}

impl Emulator {
    pub fn new(bin_data: Vec<u8>) -> Emulator {
        let mut bus = DynBus::new();
        let elf = object::File::parse(&*bin_data).expect("parsing");

        let section = elf.section_by_name(".text").expect("need text section");
        let start = section.address() as usize;
        let pc = start;

        let ram = Ram::new();
        ram.write(0 + 0x100000, section.data().expect("data").to_vec());
        bus.map(
            ram,
            Range {
                start: pc - 0x100000,
                end: pc + 0x200000,
            },
        );

        let bus = Arc::new(bus);

        let mut hart = Hart::new(0, pc, bus.clone());

        hart.set_register(treg("sp"), (pc + 0x100000) as u64);

        let (sender, receiver) = mpsc::channel();

        thread::spawn(move || {
            Emulator::run_hart(hart, receiver);
        });

        Self { bus, sender }
    }

    fn run_hart(mut hart: Hart<DynBus>, receiver: Receiver<EmulationCommand>) {
        let mut breakpoints = Vec::new();
        let mut mode = ExecutionMode::Halt;

        loop {
            match mode {
                ExecutionMode::Continue | ExecutionMode::Step => Emulator::handle_cmd(
                    &mut hart,
                    &mut breakpoints,
                    &mut mode,
                    receiver.try_recv(),
                ),
                ExecutionMode::Halt => Emulator::handle_cmd(
                    &mut hart,
                    &mut breakpoints,
                    &mut mode,
                    receiver.recv().map_err(|_e| TryRecvError::Disconnected),
                ),
            }

            if let Some(bp) = breakpoints.iter().find(|x| **x == hart.get_pc()) {
                // breakpoint found, stopping execution
                eprintln!("breakpoint found: {:x}", bp);
                mode = ExecutionMode::Halt;
            };

            match mode {
                ExecutionMode::Continue => {}
                ExecutionMode::Halt => continue,
                ExecutionMode::Step => mode = ExecutionMode::Halt,
            }

            match hart.tick() {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("exited at: {:?}", e);
                    break;
                }
            }
        }
    }

    fn handle_cmd(
        hart: &mut Hart<DynBus>,
        breakpoints: &mut Vec<usize>,
        mode: &mut ExecutionMode,
        cmd: Result<EmulationCommand, TryRecvError>,
    ) {
        match cmd {
            Ok(cmd) => match cmd {
                EmulationCommand::AddBreakpoint(addr) => {
                    breakpoints.push(addr);
                }
                EmulationCommand::RemoveBreakpoint(addr) => breakpoints.retain(|bp| *bp != addr),
                EmulationCommand::ReadMemory(_sender, _addr, _len) => {}
                EmulationCommand::WriteMemory(_addr, _data) => {}
                EmulationCommand::Resume => {}
                EmulationCommand::ReadRegisters(sender) => {
                    let mut registers = vec![hart.get_pc() as u64];
                    registers.extend_from_slice(&hart.get_registers());
                    sender.send(registers).expect("disco");
                }
                EmulationCommand::SetRegisters(regs) => {
                    hart.set_pc(regs[0] as usize);
                    for i in 0..=31 {
                        hart.set_register(i, regs[(i + 1) as usize]);
                    }
                }
                EmulationCommand::SetResumeAction(m) => {
                    *mode = m;
                }
                EmulationCommand::ClearResumeAction => {
                    *mode = ExecutionMode::Continue;
                }
            },
            Err(e) => match e {
                TryRecvError::Empty => {}
                TryRecvError::Disconnected => eprintln!("failed receiving cmd: {}", e),
            },
        }
    }
}

impl target::Target for Emulator {
    type Arch = gdbstub_arch::riscv::Riscv64;
    type Error = Fault;

    fn base_ops(&mut self) -> BaseOps<'_, Self::Arch, Self::Error> {
        MultiThread(self)
    }

    fn support_breakpoints(
        &mut self,
    ) -> Option<target::ext::breakpoints::BreakpointsOps<'_, Self>> {
        Some(self)
    }
}

impl target::ext::base::multithread::MultiThreadBase for Emulator {
    fn read_registers(
        &mut self,
        regs: &mut <Self::Arch as Arch>::Registers,
        tid: Tid,
    ) -> TargetResult<(), Self> {
        let (sender, receiver) = mpsc::channel();
        self.sender
            .send(EmulationCommand::ReadRegisters(sender))
            .expect("disco");
        let registers = receiver.recv().expect("disco");
        regs.pc = registers[0];
        for i in 0..=31 {
            regs.x[i] = registers[i + 1];
        }

        eprintln!("reading registers from tid:{} regs: {:?}", tid, regs);
        Ok(())
    }

    fn write_registers(
        &mut self,
        regs: &<Self::Arch as Arch>::Registers,
        tid: Tid,
    ) -> TargetResult<(), Self> {
        let mut registers = vec![regs.pc];
        registers.extend_from_slice(&regs.x);
        self.sender
            .send(EmulationCommand::SetRegisters(registers))
            .expect("disco");

        eprintln!("writing registers to tid:{} regs: {:?}", tid, regs);
        Ok(())
    }

    fn read_addrs(
        &mut self,
        start_addr: <Self::Arch as Arch>::Usize,
        data: &mut [u8],
        tid: Tid,
    ) -> TargetResult<(), Self> {
        self.bus.read(start_addr as usize, data).unwrap_or_default();

        eprintln!("reading from tid:{} addr {:x}: {:?}", tid, start_addr, data);
        Ok(())
    }

    fn write_addrs(
        &mut self,
        start_addr: <Self::Arch as Arch>::Usize,
        data: &[u8],
        tid: Tid,
    ) -> TargetResult<(), Self> {
        self.bus.write(start_addr as usize, data).expect("asdf");

        eprintln!("writing to tid:{} addr {:x}: {:?}", tid, start_addr, data);
        Ok(())
    }

    fn list_active_threads(
        &mut self,
        thread_is_active: &mut dyn FnMut(Tid),
    ) -> Result<(), Self::Error> {
        eprintln!("registering active thread: {}", 1);
        thread_is_active(Tid::new(1).unwrap());
        Ok(())
    }

    fn support_resume(
        &mut self,
    ) -> Option<target::ext::base::multithread::MultiThreadResumeOps<'_, Self>> {
        Some(self)
    }
}

impl target::ext::base::multithread::MultiThreadResume for Emulator {
    fn resume(&mut self) -> Result<(), Self::Error> {
        eprintln!("> resume");
        self.sender.send(EmulationCommand::Resume).expect("disco");
        Ok(())
    }

    fn clear_resume_actions(&mut self) -> Result<(), Self::Error> {
        eprintln!("> clear_resume_actions");
        self.sender
            .send(EmulationCommand::ClearResumeAction)
            .expect("disco");
        Ok(())
    }

    fn set_resume_action_continue(
        &mut self,
        _tid: Tid,
        signal: Option<gdbstub::common::Signal>,
    ) -> Result<(), Self::Error> {
        if signal.is_some() {
            // No support for resuming via signals
            return Err(Fault::Unimplemented);
        }

        eprintln!("> set_resume_action_continue");
        self.sender
            .send(EmulationCommand::SetResumeAction(ExecutionMode::Continue))
            .expect("disco");
        Ok(())
    }
}

impl target::ext::breakpoints::Breakpoints for Emulator {
    fn support_sw_breakpoint(
        &mut self,
    ) -> Option<target::ext::breakpoints::SwBreakpointOps<'_, Self>> {
        Some(self)
    }
}

impl target::ext::breakpoints::SwBreakpoint for Emulator {
    fn add_sw_breakpoint(
        &mut self,
        addr: <Self::Arch as Arch>::Usize,
        kind: <Self::Arch as Arch>::BreakpointKind,
    ) -> TargetResult<bool, Self> {
        eprintln!("adding breakpoint on {:x}({})", addr, kind);

        self.sender
            .send(EmulationCommand::AddBreakpoint(addr as usize))
            .expect("disco");

        Ok(true)
    }

    fn remove_sw_breakpoint(
        &mut self,
        addr: <Self::Arch as Arch>::Usize,
        kind: <Self::Arch as Arch>::BreakpointKind,
    ) -> TargetResult<bool, Self> {
        eprintln!("removing breakpoint on {:x}({})", addr, kind);

        self.sender
            .send(EmulationCommand::RemoveBreakpoint(addr as usize))
            .expect("disco");

        Ok(true)
    }
}
