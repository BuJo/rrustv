mod bus;
mod csr;
mod dts;
mod hart;
mod ram;
mod rom;
mod rtc;
mod see;

use std::sync::Arc;
use std::thread;
use std::{env, fs};

use crate::bus::Bus;
use crate::hart::Hart;
use crate::ram::Ram;
use crate::rom::Rom;
use crate::rtc::Rtc;

fn main() {
    let args: Vec<String> = env::args().collect();
    let threads = args.get(1).and_then(|x| x.parse::<u32>().ok()).unwrap_or(1);

    let text = fs::read("target/target.text").expect("no .text");
    let dtb = dts::load();

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
            let mut m = Hart::new(id, bus);
            for _ in 0..100 {
                if !m.tick() {
                    break;
                }
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("hart failed")
    }
}
