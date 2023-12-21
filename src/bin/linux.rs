use std::ops::Range;
use std::sync::Arc;
use std::{env, fs};

use log::error;
use object::{Object, ObjectSection};

use rriscv::dynbus::DynBus;
use rriscv::hart::Hart;
use rriscv::ram::Ram;
use rriscv::reg::treg;
use rriscv::rom::Rom;
use rriscv::rtc::Rtc;
use rriscv::uart::Uart8250;
use rriscv::{clint, dt, plic, virtio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let image_file = args.get(1).expect("expect image file");
    let disk_file = args.get(2).expect("expect disc file");

    let bin_data = fs::read(image_file).expect("file");
    let elf = object::File::parse(&*bin_data).expect("parsing");

    let mut bus = DynBus::new();
    let ram = Ram::new();
    let pc = elf.entry() as usize;

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
            end: 0x88000000,
        },
    );

    let rtc = Rtc::new();
    bus.map(rtc, 0x4000..0x4020);

    let console = Uart8250::new();
    bus.map(console, 0x10000000..0x10000010);

    // Add a rom at 0 to catch 0x00 reads
    let rom = Rom::new(vec![]);
    bus.map(rom, 0x0..0x1000);

    // virtio block device vda
    let vda = virtio::Device::new_block_device(disk_file);
    bus.map(vda, 0x10001000..0x10002000);

    let clint = clint::Clint::new();
    bus.map(clint, 0x2000000..0x2010000);

    let plic = plic::Plic::new();
    bus.map(plic, 0xc000000..0xc600000);

    let device_tree = dt::load("linux");
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

    loop {
        match hart.tick() {
            Ok(_) => {}
            Err(err) => error!("err: {:?}", err),
        }
    }
}
