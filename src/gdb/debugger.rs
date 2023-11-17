use crate::gdb::emulator::Emulator;
use gdbstub::conn::ConnectionExt;
use gdbstub::stub::state_machine::GdbStubStateMachine;
use gdbstub::stub::{DisconnectReason, GdbStub, GdbStubError, MultiThreadStopReason};
use std::net::{TcpListener, TcpStream};

pub struct Debugger<'a> {
    emu: &'a mut Emulator,
}

impl<'a> Debugger<'a> {
    pub fn new(emu: &'a mut Emulator) -> Self {
        Self { emu }
    }

    pub fn wait_for_tcp(port: u16) -> Result<TcpStream, Box<dyn std::error::Error>> {
        let sockaddr = format!("127.0.0.1:{}", port);
        eprintln!("Waiting for a GDB connection on {:?}...", sockaddr);

        let sock = TcpListener::bind(sockaddr)?;
        let (stream, addr) = sock.accept()?;
        eprintln!("Debugger connected from {}", addr);

        Ok(stream)
    }

    pub fn run(&mut self, conn: TcpStream) {
        let gdb = GdbStub::new(conn);

        let mut gdb = gdb
            .run_state_machine(self.emu)
            .expect("Emulator implemented incorrectly");
        let res = loop {
            gdb = match gdb {
                GdbStubStateMachine::Idle(mut gdb) => {
                    let byte = match gdb.borrow_conn().read() {
                        Ok(byte) => byte,
                        Err(_) => break Err(GdbStubError::UnsupportedStopReason),
                    };
                    match gdb.incoming_data(self.emu, byte) {
                        Ok(gdb) => gdb,
                        Err(e) => break Err(e),
                    }
                }
                GdbStubStateMachine::Running(gdb) => {
                    match gdb.report_stop(self.emu, MultiThreadStopReason::DoneStep) {
                        Ok(gdb) => gdb,
                        Err(e) => break Err(e),
                    }
                }
                GdbStubStateMachine::CtrlCInterrupt(gdb) => {
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
