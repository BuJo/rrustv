use object::{Object, ObjectSection};
use rriscv::dynbus::DynBus;
use rriscv::gdb::debugger::Debugger;
use rriscv::gdb::emulator::Emulator;
use rriscv::hart::Hart;
use rriscv::ram::Ram;
use rriscv::rtc::Rtc;
use std::net::TcpStream;
use std::ops::Range;
use std::sync::Arc;
use std::{env, fs};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let image_file = args.get(1).expect("expect image file");

    let bin_data = fs::read(image_file).expect("file");
    let elf = object::File::parse(&*bin_data).expect("parsing");

    let mut bus = DynBus::new();
    let ram = Ram::new();
    let mut pc = elf.entry() as usize;

    for section in elf.sections() {
        let name = section.name().expect("section name");
        if name.contains("data") || name.contains("text") {
            let start = section.address() as usize;
            if let Ok(data) = section.uncompressed_data() {
                ram.write(start - pc, data.to_vec());
            }
        }
    }

    bus.map(
        ram,
        Range {
            start: pc,
            end: 0xFFFFFFFFFFFFFFFF,
        },
    );

    let rtc = Rtc::new();
    bus.map(rtc, 0x4000..0x4020);

    let bus = Arc::new(bus);

    let hart = Hart::new(0, pc, bus.clone());

    let mut emu = Emulator::new_plain(hart, bus);

    let conn: TcpStream = Debugger::wait_for_tcp(9001)?;
    let mut gdb = Debugger::new(&mut emu);

    gdb.run(conn);

    Ok(())
}
