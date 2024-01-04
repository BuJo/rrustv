use std::sync::Arc;
use std::{env, fs};

use log::{info, warn};

use rriscv::bus::DynBus;
use rriscv::hart::Hart;
use rriscv::ram::Ram;
use rriscv::rtc::Rtc;

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let image_file = args.get(1).expect("expect image file");

    let bus = DynBus::new();

    let bin_data = fs::read(image_file).expect("file");

    let ram = Ram::new();
    ram.write(0, bin_data);
    bus.map(ram, 0x80000000..0xFFFFFFFF);

    let rtc = Rtc::new();
    bus.map(rtc, 0x4000..0x4020);

    let bus = Arc::new(bus);

    let mut m = Hart::new(0, 0x80000000, bus.clone());
    let mut i = 0;
    loop {
        match m.tick() {
            Ok(_) => {}
            Err(e) => {
                info!("exited at: {} ({:?})", i, e);
                break;
            }
        }

        if i >= 1_000_000 {
            warn!("endless, killing");
            break;
        }
        i += 1;
    }
}
