use rriscv::gdb::debugger::Debugger;
use rriscv::gdb::emulator::Emulator;
use std::net::TcpStream;
use std::{env, fs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let image_file = args.get(1).expect("expect image file");
    let bin_data = fs::read(image_file).expect("file");

    let mut emu = Emulator::new(bin_data);

    let conn: TcpStream = Debugger::wait_for_tcp(9001)?;
    let mut gdb = Debugger::new(&mut emu);

    gdb.run(conn);

    Ok(())
}
