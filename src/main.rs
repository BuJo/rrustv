use std::{env, fs};
use std::sync::Arc;
use std::thread;

use rriscv::bus::Bus;
use rriscv::dt;
use rriscv::hart::Hart;
use rriscv::ram::Ram;
use rriscv::rom::Rom;
use rriscv::rtc::Rtc;

fn main() {
    let args: Vec<String> = env::args().collect();
    let threads = args.get(1).and_then(|x| x.parse::<u32>().ok()).unwrap_or(1);

    let text = fs::read("target/target.text").expect("no .text");
    let dtb = dt::load();

    let rom = Rom::new(text);
    let ram = Ram::new();
    ram.write(0, dtb);

    let rtc = Rtc::new();

    let bus = Arc::new(Bus::new(rom, ram, rtc));

    let mut handles = vec![];

    for id in 0..threads {
        let bus = bus.clone();

        let handle = thread::spawn(move || {
            eprintln!("[{}] hart spawned", id);
            let mut m = Hart::new(id, 0, bus);
            for i in 0..100 {
                match m.tick() {
                    Ok(_) => {}
                    Err(e) => {
                        eprintln!("exited at: {} ({:?})", i, e);
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
