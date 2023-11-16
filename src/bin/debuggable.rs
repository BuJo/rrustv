use std::net::{TcpListener, TcpStream};
use std::ops::Range;
use std::sync::Arc;
use std::{env, fs};

use gdbstub::arch::Arch;
use gdbstub::common::Tid;
use gdbstub::conn::ConnectionExt;
use gdbstub::stub::state_machine::GdbStubStateMachine;
use gdbstub::stub::{DisconnectReason, GdbStub, GdbStubError, MultiThreadStopReason};
use gdbstub::target;
use gdbstub::target::ext::base::BaseOps;
use gdbstub::target::ext::base::BaseOps::MultiThread;
use gdbstub::target::TargetResult;
use object::{Object, ObjectSection};
use rriscv::csr::MHARTID;

use rriscv::dynbus::DynBus;
use rriscv::hart::Hart;
use rriscv::ram::Ram;
use rriscv::reg::treg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let image_file = args.get(1).expect("expect image file");
    let bin_data = fs::read(image_file).expect("file");

    let mut emu = Emulator::new(bin_data);

    let conn: TcpStream = wait_for_tcp(9001)?;
    let mut gdb = Debugger::new(&mut emu);

    gdb.run(conn);

    Ok(())
}

struct Debugger<'a> {
    emu: &'a mut Emulator,
}
impl<'a> Debugger<'a> {
    fn new(emu: &'a mut Emulator) -> Self {
        Self { emu }
    }

    fn run(&mut self, conn: TcpStream) {
        let gdb = GdbStub::new(conn);

        let mut gdb = gdb.run_state_machine(self.emu).expect("ok");
        let res = loop {
            gdb = match gdb {
                GdbStubStateMachine::Idle(mut gdb) => {
                    let byte = match gdb.borrow_conn().read() {
                        Ok(byte) => byte,
                        Err(e) => break Err(GdbStubError::UnsupportedStopReason),
                    };
                    match gdb.incoming_data(self.emu, byte) {
                        Ok(gdb) => gdb,
                        Err(e) => break Err(e),
                    }
                }
                GdbStubStateMachine::Running(mut gdb) => {
                    match gdb.report_stop(self.emu, MultiThreadStopReason::DoneStep) {
                        Ok(gdb) => gdb,
                        Err(e) => break Err(e),
                    }
                }
                GdbStubStateMachine::CtrlCInterrupt(mut gdb) => {
                    match gdb.interrupt_handled(self.emu, None::<MultiThreadStopReason<u64>>) {
                        Ok(gdb) => gdb,
                        Err(e) => break Err(e),
                    }
                }
                GdbStubStateMachine::Disconnected(gdb) => break Ok(gdb.get_reason()),
            }
        };

        match res {
            Ok(disconnect_reason) => match disconnect_reason {
                DisconnectReason::Disconnect => println!("GDB Disconnected"),
                DisconnectReason::TargetExited(rc) => println!("Target exited: {}", rc),
                DisconnectReason::TargetTerminated(signal) => println!("Target halted: {}", signal),
                DisconnectReason::Kill => println!("GDB sent a kill command"),
            },
            Err(GdbStubError::TargetError(e)) => {
                println!("Target raised a fatal error: {:?}", e);
            }
            Err(e) => {
                println!("gdbstub internal error: {:?}", e);
            }
        }
    }
}

fn wait_for_tcp(port: u16) -> Result<TcpStream, Box<dyn std::error::Error>> {
    let sockaddr = format!("127.0.0.1:{}", port);
    eprintln!("Waiting for a GDB connection on {:?}...", sockaddr);

    let sock = TcpListener::bind(sockaddr)?;
    let (stream, addr) = sock.accept()?;
    eprintln!("Debugger connected from {}", addr);

    Ok(stream)
}

struct Emulator {
    bus: Arc<DynBus>,
    hart: Hart<DynBus>,

    breakpoints: Vec<usize>,
}

impl Emulator {
    fn new(bin_data: Vec<u8>) -> Emulator {
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

        Self { bus, hart }
    }

    fn run(&mut self) {
        let mut i = 0;

        loop {
            match self.hart.tick() {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("exited at: {} ({:?})", i, e);
                    break;
                }
            }
            i += 1;
        }
    }
}

impl target::Target for Emulator {
    type Arch = gdbstub_arch::riscv::Riscv64;
    type Error = ();

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
        regs.pc = self.hart.get_pc() as u64;
        for i in 0..=31 {
            regs.x[i] = self.hart.get_register(i as u8);
        }

        eprintln!("reading registers from tid:{} regs: {:?}", tid, regs);
        Ok(())
    }

    fn write_registers(
        &mut self,
        regs: &<Self::Arch as Arch>::Registers,
        tid: Tid,
    ) -> TargetResult<(), Self> {
        self.hart.set_pc(regs.pc as usize);
        for i in 0..=31 {
            self.hart.set_register(i, regs.x[i as usize]);
        }

        eprintln!("writing registers to tid:{} regs: {:?}", tid, regs);
        Ok(())
    }

    fn read_addrs(
        &mut self,
        start_addr: <Self::Arch as Arch>::Usize,
        data: &mut [u8],
        tid: Tid,
    ) -> TargetResult<(), Self> {
        self.bus.read(start_addr as usize, data).expect("asdf");

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
        let hartid = self.hart.get_csr(MHARTID) as usize;

        eprintln!("registering active thread: {}", hartid + 1);
        thread_is_active(Tid::new(hartid + 1).unwrap());
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
        Ok(())
    }

    fn clear_resume_actions(&mut self) -> Result<(), Self::Error> {
        eprintln!("> clear_resume_actions");
        Ok(())
    }

    fn set_resume_action_continue(
        &mut self,
        _tid: Tid,
        _signal: Option<gdbstub::common::Signal>,
    ) -> Result<(), Self::Error> {
        eprintln!("> set_resume_action_continue");
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

        self.breakpoints.push(addr as usize);

        Ok(true)
    }

    fn remove_sw_breakpoint(
        &mut self,
        addr: <Self::Arch as Arch>::Usize,
        kind: <Self::Arch as Arch>::BreakpointKind,
    ) -> TargetResult<bool, Self> {
        eprintln!("removing breakpoint on {:x}({})", addr, kind);

        self.breakpoints.retain(|bp| *bp != addr as usize);

        Ok(true)
    }
}
