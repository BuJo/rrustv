use std::net::{TcpListener, TcpStream};

use gdbstub::stub::{DisconnectReason, GdbStub, GdbStubError};

use crate::gdb::emulator::Emulator;
use crate::gdb::runner::GdbBlockingEventLoop;

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

        match gdb.run_blocking::<GdbBlockingEventLoop>(&mut self.emu) {
            Ok(disconnect_reason) => match disconnect_reason {
                DisconnectReason::Disconnect => {
                    println!("Client disconnected")
                }
                DisconnectReason::TargetExited(code) => {
                    println!("Target exited with code {}", code)
                }
                DisconnectReason::TargetTerminated(sig) => {
                    println!("Target terminated with signal {}", sig)
                }
                DisconnectReason::Kill => println!("GDB sent a kill command"),
            },
            Err(GdbStubError::TargetError(e)) => {
                println!("target encountered a fatal error: {}", e)
            }
            Err(e) => {
                println!("gdbstub encountered a fatal error: {}", e)
            }
        }
    }
}
