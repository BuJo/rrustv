mod csr;
mod hart;
mod ram;
mod see;

use crate::hart::Hart;
use crate::ram::Ram;
use std::sync::Arc;
use std::thread;
use std::{env, fs};

fn main() {
    let args: Vec<String> = env::args().collect();
    let threads = args.get(1).and_then(|x| x.parse::<i32>().ok()).unwrap_or(1);

    let text = fs::read("target/target.text").expect("no .text");
    let data = fs::read("target/target.data").expect("no .data");

    let ram = Arc::new(Ram::new(text));
    ram.write(0x1000, data);

    let mut handles = vec![];

    for _ in 0..threads {
        let ram = Arc::clone(&ram);
        let handle = thread::spawn(move || {
            let mut m = Hart::new(ram);
            for _ in 0..100 {
                if !m.tick() {
                    break;
                }
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().expect("Thread exit");
    }
}
