use std::ops::Range;
use std::sync::Arc;
use std::{env, fs};

use rriscv::dynbus::DynBus;
use rriscv::hart::Hart;
use rriscv::ram::Ram;
use rriscv::rom::Rom;
use rriscv::rtc::Rtc;

fn main() {
    let args: Vec<String> = env::args().collect();
    let elf_file = args.get(1).expect("expect elf file");

    let mut bus = DynBus::new();

    let bin_data = fs::read(elf_file).expect("file");

    let rom = Rom::new(bin_data.to_vec());
    bus.map(
        rom,
        Range {
            start: 0,
            end: bin_data.len(),
        },
    );

    let ram = Ram::new();
    bus.map(ram, 0x8000000..0xFFFFFFFF);

    let rtc = Rtc::new();
    bus.map(rtc, 0x4000..0x4020);

    let bus = Arc::new(bus);

    let mut m = Hart::new(0, 0, bus.clone());
    let mut i = 0;
    loop {
        match m.tick() {
            Ok(_) => {}
            Err(e) => {
                eprintln!("exited at: {} ({:?})", i, e);
                break;
            }
        }

        if i >= 1_000_000 {
            eprintln!("endless, killing");
            break;
        }
        i += 1;
    }
}
