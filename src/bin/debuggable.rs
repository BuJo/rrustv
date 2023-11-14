use std::{env, fs};
use std::net::{TcpListener, TcpStream};
use std::ops::Range;
use std::sync::Arc;

use gdbstub::arch::Arch;
use gdbstub::common::Tid;
use gdbstub::conn::ConnectionExt;
use gdbstub::stub::{DisconnectReason, GdbStub, GdbStubError, MultiThreadStopReason};
use gdbstub::stub::state_machine::GdbStubStateMachine;
use gdbstub::target;
use gdbstub::target::ext::base::BaseOps;
use gdbstub::target::ext::base::BaseOps::MultiThread;
use gdbstub::target::TargetResult;
use object::{Object, ObjectSection};

use rriscv::dynbus::DynBus;
use rriscv::hart::Hart;
use rriscv::ram::Ram;
use rriscv::reg::treg;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let image_file = args.get(1).expect("expect image file");
    let bin_data = fs::read(image_file).expect("file");

    let conn: TcpStream = wait_for_tcp(9001)?;
    let gdb: GdbStub<Emulator, TcpStream> = GdbStub::new(conn);

    let mut emu = Emulator::new(bin_data);

    let mut gdb = gdb.run_state_machine(&mut emu)?;

    let res = loop {
        gdb = match gdb {
            GdbStubStateMachine::Idle(mut gdb) => {
                let byte = gdb.borrow_conn().read()?;
                match gdb.incoming_data(&mut emu, byte) {
                    Ok(gdb) => gdb,
                    Err(e) => break Err(e),
                }
            }
            GdbStubStateMachine::Running(gdb) => {
                match gdb.report_stop(&mut emu, MultiThreadStopReason::DoneStep) {
                    Ok(gdb) => gdb,
                    Err(e) => break Err(e),
                }
            }
            GdbStubStateMachine::CtrlCInterrupt(gdb) => {
                match gdb.interrupt_handled(&mut emu, None::<MultiThreadStopReason<u64>>) {
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
            DisconnectReason::TargetExited(_) => println!("Target exited"),
            DisconnectReason::TargetTerminated(_) => println!("Target halted"),
            DisconnectReason::Kill => println!("GDB sent a kill command"),
        },
        Err(GdbStubError::TargetError(_e)) => {
            println!("Target raised a fatal error");
        }
        Err(_e) => {
            println!("gdbstub internal error");
        }
    }

    Ok(())
}

fn wait_for_tcp(port: u16) -> Result<TcpStream, Box<dyn std::error::Error>> {
    let sockaddr = format!("127.0.0.1:{}", port);
    eprintln!("Waiting for a GDB connection on {:?}...", sockaddr);

    let sock = TcpListener::bind(sockaddr)?;
    let (stream, addr) = sock.accept()?;
    eprintln!("Debugger connected from {}", addr);

    Ok(stream)
}

struct Emulator  {
    hart: Hart<DynBus>
}

impl Emulator {
    fn new(bin_data: Vec<u8>) -> Emulator {

        let mut bus = DynBus::new();
        let elf = object::File::parse(&*bin_data).expect("parsing");

        let section = elf.section_by_name(".text").expect("need text section");
        let start = section.address() as usize;
        let pc = start;

        let ram = Ram::new();
        ram.write(0, section.data().expect("data").to_vec());
        bus.map(ram, Range { start: pc, end: pc+0x100000 });

        let bus = Arc::new(bus);

        let mut hart = Hart::new(0, pc, bus.clone());

        hart.set_register(treg("sp"), (pc+0x100000) as u64);

        Self {
            hart
        }
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

    #[inline(always)]
    fn support_breakpoints(&mut self) -> Option<target::ext::breakpoints::BreakpointsOps<'_, Self>> {
        Some(self)
    }
}

impl target::ext::base::multithread::MultiThreadBase for Emulator {
    fn read_registers(&mut self, regs: &mut <Self::Arch as Arch>::Registers, tid: Tid) -> TargetResult<(), Self> {
        eprintln!("reading registers from tid:{} regs: {:?}", tid, regs);
        Ok(())
    }

    fn write_registers(&mut self, regs: &<Self::Arch as Arch>::Registers, tid: Tid) -> TargetResult<(), Self> {
        eprintln!("writing registers to tid:{} regs: {:?}", tid, regs);
        Ok(())
    }

    fn read_addrs(&mut self, start_addr: <Self::Arch as Arch>::Usize, data: &mut [u8], tid: Tid) -> TargetResult<(), Self> {
        eprintln!("reading from tid:{} addr {:x}: {:?}", tid, start_addr, data);
        Ok(())
    }

    fn write_addrs(&mut self, start_addr: <Self::Arch as Arch>::Usize, data: &[u8], tid: Tid) -> TargetResult<(), Self> {
        eprintln!("writing to tid:{} addr {:x}: {:?}", tid, start_addr, data);
        Ok(())
    }

    fn list_active_threads(&mut self, thread_is_active: &mut dyn FnMut(Tid)) -> Result<(), Self::Error> {
        eprintln!("registering active threads");
        thread_is_active(Tid::new(1).unwrap());
        Ok(())
    }
}

impl target::ext::breakpoints::Breakpoints for Emulator {
    #[inline(always)]
    fn support_sw_breakpoint(&mut self) -> Option<target::ext::breakpoints::SwBreakpointOps<'_, Self>> {
        Some(self)
    }
}

impl target::ext::breakpoints::SwBreakpoint for Emulator {
    fn add_sw_breakpoint(&mut self, addr: <Self::Arch as Arch>::Usize, kind: <Self::Arch as Arch>::BreakpointKind) -> TargetResult<bool, Self> {
        eprintln!("adding breakpoint on {:x}({})", addr, kind);
        Ok(true)
    }

    fn remove_sw_breakpoint(&mut self, addr: <Self::Arch as Arch>::Usize, kind: <Self::Arch as Arch>::BreakpointKind) -> TargetResult<bool, Self> {
        eprintln!("adding breakpoint on {:x}({})", addr, kind);
        Ok(true)
    }
}
