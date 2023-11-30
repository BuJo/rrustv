use std::collections::HashMap;
use std::net::TcpStream;
use std::ops::Range;
use std::sync::{Arc, mpsc, Once};
use std::sync::mpsc::{Receiver, Sender, SendError, TryRecvError};
use std::thread;

use gdbstub::common::Tid;
use gdbstub::conn::{Connection, ConnectionExt};
use gdbstub::stub::MultiThreadStopReason;
use gdbstub::stub::run_blocking::Event;
use gdbstub::target;
use gdbstub::target::ext::base::BaseOps;
use gdbstub::target::ext::base::BaseOps::MultiThread;
use object::{Object, ObjectSection};

use crate::csr;
use crate::dynbus::DynBus;
use crate::hart::Hart;
use crate::plic::Fault;
use crate::ram::Ram;
use crate::reg::treg;

pub(crate) enum EmulationCommand {
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

pub(crate) enum ExecutionMode {
    Continue,
    Halt,
    Step,
    Pause,
}

pub struct Emulator {
    pub(crate) bus: Arc<DynBus>,
    pub(crate) sender: Sender<EmulationCommand>,
    state_receiver: Receiver<Event<MultiThreadStopReason<u64>>>,
    byte_sender: Sender<Event<MultiThreadStopReason<u64>>>,
    start_conn_reader: Once,
    gdb_connections: HashMap<TcpStream, bool>,
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

        let (state_sender, state_receiver) = mpsc::channel();
        let (sender, receiver) = mpsc::channel();

        let byte_sender = state_sender.clone();
        thread::spawn(move || {
            Emulator::run_hart(hart, receiver, state_sender);
        });

        Self {
            bus,
            sender,
            state_receiver,
            byte_sender,
            start_conn_reader: Once::new(),
            gdb_connections: HashMap::new(),
        }
    }

    pub fn new_plain(hart: Hart<DynBus>, bus: Arc<DynBus>) -> Emulator {
        let (state_sender, state_receiver) = mpsc::channel();
        let (sender, receiver) = mpsc::channel();

        let byte_sender = state_sender.clone();
        thread::spawn(move || {
            Emulator::run_hart(hart, receiver, state_sender);
        });

        Self {
            bus,
            sender,
            state_receiver,
            byte_sender,
            start_conn_reader: Once::new(),
            gdb_connections: HashMap::new(),
        }
    }

    fn run_hart(
        mut hart: Hart<DynBus>,
        receiver: Receiver<EmulationCommand>,
        state_sender: Sender<Event<MultiThreadStopReason<u64>>>,
    ) {
        let tid = Tid::new(hart.get_csr(csr::MHARTID) as usize + 1).unwrap();
        let mut breakpoints = Vec::new();
        let mut mode = ExecutionMode::Pause; // start harts paused

        loop {
            match mode {
                ExecutionMode::Continue | ExecutionMode::Step => Emulator::handle_cmd(
                    &mut hart,
                    &mut breakpoints,
                    &mut mode,
                    receiver.try_recv(),
                ),
                ExecutionMode::Pause => Emulator::handle_cmd(
                    &mut hart,
                    &mut breakpoints,
                    &mut mode,
                    receiver.recv().map_err(|_e| TryRecvError::Disconnected),
                ),
                ExecutionMode::Halt => {}
            }

            if let Some(bp) = breakpoints.iter().find(|x| **x == hart.get_pc()) {
                // breakpoint found, stopping execution
                eprintln!("breakpoint found: {:x}", bp);
                mode = ExecutionMode::Pause;
                let snd = state_sender
                    .send(Event::TargetStopped(MultiThreadStopReason::SwBreak(tid)));
                match snd {
                    Ok(_) => {}
                    Err(_) => {
                        // we must be disconnected, assume no debugging
                        breakpoints.clear();
                        mode = ExecutionMode::Continue;
                    }
                }
            };

            match mode {
                ExecutionMode::Continue => {}
                ExecutionMode::Halt => {
                    state_sender
                        .send(Event::TargetStopped(MultiThreadStopReason::Exited(0)))
                        .expect("disco");
                }
                ExecutionMode::Pause => continue,
                ExecutionMode::Step => mode = ExecutionMode::Pause,
            }

            match hart.tick() {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("exited at: {:?}", e);
                    state_sender
                        .send(Event::TargetStopped(MultiThreadStopReason::Exited(1)))
                        .expect("disco");
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

// Implements interface used by runner
impl Emulator {
    pub(crate) fn stop_in_response_to_ctrl_c_interrupt(&self) -> Result<(), Fault> {
        self.sender
            .send(EmulationCommand::SetResumeAction(ExecutionMode::Pause))
            .expect("disco");
        Ok(())
    }

    pub(crate) fn read_stop_event(
        &self,
        conn: &mut TcpStream,
    ) -> Event<MultiThreadStopReason<u64>> {
        if !self.start_conn_reader.is_completed() {
            let byte_sender = self.byte_sender.clone();
            let mut conn = conn.try_clone().expect("disco");
            self.start_conn_reader.call_once(move || {
                thread::spawn(move || {
                    loop {
                        match conn.read() {
                            Ok(data) => {
                                eprintln!("gdb data: {:x} {}", data, data as char);
                                byte_sender
                                    .send(Event::IncomingData(data))
                                    .expect("disco")
                            },
                            Err(_) => byte_sender
                                .send(Event::TargetStopped(MultiThreadStopReason::Exited(1)))
                                .expect("disco"),
                        }
                    }
                });
            });
        }

        match self.state_receiver.recv() {
            Ok(o) => o,
            Err(e) => panic!("{}", e),
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
