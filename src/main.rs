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
    let threads = args
        .get(1)
        .and_then(|x| x.parse::<usize>().ok())
        .unwrap_or(1);
    let (send_finished_thread, receive_finished_thread) = std::sync::mpsc::channel();

    let text = fs::read("target/target.text").expect("no .text");
    let data = fs::read("target/target.data").expect("no .data");

    let ram = Arc::new(Ram::new(text));
    ram.write(0x1000, data);

    let mut handles = vec![];

    for id in 0..threads {
        let ram = ram.clone();
        let send_finished_thread = send_finished_thread.clone();

        let handle = thread::spawn(move || {
            eprintln!("[{}] hart spawned", id);
            let mut m = Hart::new(id as u32, ram);
            for _ in 0..100 {
                if !m.tick() {
                    break;
                }
            }

            send_finished_thread.send(id).unwrap();
        });

        handles.push(Some(handle));
    }

    loop {
        // Check if all threads are finished
        let num_left = handles.iter().filter(|th| th.is_some()).count();
        if num_left == 0 {
            break;
        }

        // Wait until a thread is finished, then join it
        let id = receive_finished_thread.recv().unwrap();
        let join_handle = std::mem::take(&mut handles[id]).unwrap();
        match join_handle.join() {
            Ok(_) => eprintln!("[{}] hart joined.", id),
            Err(e) => eprintln!("[{}] hart joined: {:?}", id, e),
        }
    }

    println!("All hart joined.");
}
