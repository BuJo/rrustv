use object::{Object, ObjectSection};
use rriscv::dt;
use rriscv::dynbus::DynBus;
use rriscv::gdb::emu::Emulator;
use rriscv::hart::Hart;
use rriscv::ram::Ram;
use rriscv::reg::{reg, treg};
use rriscv::rom::Rom;
use rriscv::rtc::Rtc;
use std::net::{TcpListener, TcpStream};
use std::ops::Range;
use std::sync::Arc;
use std::{env, fs};
use std::cell::RefCell;
use std::rc::Rc;

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

    let device_tree = dt::load();
    let dtb_start = 0x8000;
    let dtb_end = dtb_start + device_tree.len();
    let dtb = Rom::new(device_tree);
    bus.map(dtb, 0x8000..dtb_end);

    let bus = Arc::new(bus);

    let mut hart = Hart::new(0, pc, bus.clone());

    // linux register state
    hart.set_register(treg("a0"), 0);
    hart.set_register(treg("a1"), dtb_start as u64);
    hart.set_csr(rriscv::csr::SATP, 0);

    let listener = TcpListener::bind("127.0.0.1:9001").unwrap();
    println!("Listening on port 9001");

    env_logger::init();

    let debugger = Emulator::new(hart);
    if let Ok((stream, _addr)) = listener.accept() {
        println!("Got connection");
        gdb_remote_protocol::process_packets_from(stream.try_clone().unwrap(), stream, debugger);
    }
    println!("Connection closed");

    Ok(())
}
