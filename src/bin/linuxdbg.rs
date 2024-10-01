use std::net::TcpListener;
use std::sync::Arc;
use std::{env, fs};

use log::info;
use object::{Object, ObjectSection};

use rriscv::bus::DynBus;
use rriscv::gdb::emu::Emulator;
use rriscv::hart::Hart;
use rriscv::ram::Ram;
use rriscv::reg::treg;
use rriscv::rom::Rom;
use rriscv::rtc::Rtc;
use rriscv::uart::Uart8250;
use rriscv::virtio::BlkDevice;
use rriscv::{clint, dt, plic};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let image_file = args.get(1).expect("expect image file");
    let disk_file = args.get(2).expect("expect disc file");

    let bin_data = fs::read(image_file).expect("file");
    let elf = object::File::parse(&*bin_data).expect("parsing");

    let bus = Arc::new(DynBus::new());
    let ram = Ram::new();
    let pc = elf.entry() as usize;

    info!("ELF entry: 0x{:x}", pc);

    for section in elf.sections() {
        //let name = section.name().expect("section name");
        //if name.contains("data") || name.contains("text") {
        let start = section.address() as usize;
        if start == 0 {
            continue;
        }
        if let Ok(data) = section.uncompressed_data() {
            ram.write(start - 0x80000000, data.to_vec());
        }
        //}
    }

    let s = ram.size();
    bus.map(ram, 0x80000000..(pc + s));
    info!("mapped RAM: 0x{:x} (size: {})", pc, s);

    // Add low ram
    let ram = Ram::sized(0x10000);
    bus.map(ram, 0x0..0x10000);

    let rtc = Rtc::new();
    bus.map(rtc, 0x40000..0x40020);

    let console = Uart8250::new();
    bus.map(console, 0x10000000..0x10000010);

    // virtio block device vda
    let vda = BlkDevice::new(disk_file, bus.clone());
    bus.map(vda, 0x10001000..0x10002000);

    let clint = clint::Clint::new(bus.clone(), 0x40000);
    bus.map(clint, 0x2000000..0x2010000);

    let plic = plic::Plic::new();
    bus.map(plic, 0xc000000..0xc600000);

    let device_tree = dt::load("linux");
    let dtb_start = 0x80000;
    let dtb_end = dtb_start + device_tree.len();
    let dtb = Rom::new(device_tree);
    bus.map(dtb, 0x80000..dtb_end);

    let mut hart = Hart::new(0, pc, bus.clone());

    // linux register state
    hart.set_register(treg("a0"), 0);
    hart.set_register(treg("a1"), dtb_start as u64);
    hart.set_csr(rriscv::csr::SATP, 0);

    let listener = TcpListener::bind("127.0.0.1:9001").unwrap();
    info!("Listening on port 9001");

    let debugger = Emulator::new(hart);
    if let Ok((stream, _addr)) = listener.accept() {
        info!("Got connection");
        gdb_remote_protocol::process_packets_from(stream.try_clone().unwrap(), stream, debugger);
    }
    info!("Connection closed");

    Ok(())
}
