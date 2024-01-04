use std::sync::Arc;
use std::thread;
use std::{env, fs};

use log::{debug, info};

use rriscv::bus::DynBus;
use rriscv::hart::Hart;
use rriscv::ram::Ram;
use rriscv::rom::Rom;

fn main() {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let threads = args.get(1).and_then(|x| x.parse::<u64>().ok()).unwrap_or(1);

    let text = fs::read("target/target.text").expect("no .text");

    let bus = DynBus::new();

    let rom = Rom::new(text);
    bus.map(rom, 0x0..0x1FF);

    let ram = Ram::new();
    bus.map(ram, 0x80000000..0x88000000);

    let bus = Arc::new(bus);

    let mut handles = vec![];

    for id in 0..threads {
        let bus = bus.clone();

        let handle = thread::spawn(move || {
            debug!("[{}] hart spawned", id);
            let mut m = Hart::new(id, 0, bus);
            for i in 0..100 {
                match m.tick() {
                    Ok(_) => {}
                    Err(e) => {
                        info!("exited at: {} ({:?})", i, e);
                        break;
                    }
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("hart failed")
    }
}
