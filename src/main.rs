mod bus;
mod csr;
mod hart;
mod ram;
mod see;

use crate::bus::Bus;
use crate::hart::Hart;
use crate::ram::Ram;
use std::sync::Arc;
use std::thread;
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();
    let threads = args.get(1).and_then(|x| x.parse::<u32>().ok()).unwrap_or(1);

    let text = fs::read("target/target.text").expect("no .text");
    let data = fs::read("target/target.data").expect("no .data");

    let rom = Ram::new(text);
    rom.write(0x1000, data);

    let bus = Arc::new(Bus::new(rom));

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
