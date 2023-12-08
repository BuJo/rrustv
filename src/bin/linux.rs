use std::{env, fs};
use std::net::TcpListener;
use std::ops::Range;
use std::sync::Arc;

use log::{info, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::Config;
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use object::{Object, ObjectSection};

use rriscv::dt;
use rriscv::dynbus::DynBus;
use rriscv::gdb::emu::Emulator;
use rriscv::hart::Hart;
use rriscv::ram::Ram;
use rriscv::reg::treg;
use rriscv::rom::Rom;
use rriscv::rtc::Rtc;
use rriscv::uart8250::Uart8250;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stdout = ConsoleAppender::builder().build();
    let debug = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build("debug.log")
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("debug", Box::new(debug)))
        .logger(Logger::builder()
            .appender("debug")
            .additive(false)
            .build("rriscv", LevelFilter::Trace))
        .build(Root::builder().appender("stdout").build(LevelFilter::Warn))
        .unwrap();

    let _ = log4rs::init_config(config).unwrap();

    let args: Vec<String> = env::args().collect();
    let image_file = args.get(1).expect("expect image file");

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

    let device_tree = dt::load();
    let dtb_start = 0x8000;
    let dtb_end = dtb_start + device_tree.len();
    let dtb = Rom::new(device_tree);
    bus.map(dtb, 0x8000..dtb_end);

    let console = Uart8250::new();
    bus.map(console, 0x10000000..0x100000FF);

    // Add a rom at 0 to catch 0x00 reads
    let rom = Rom::new(vec![]);
    bus.map(rom, 0x0..0x1000);

    let bus = Arc::new(bus);

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
