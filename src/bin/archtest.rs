use std::{env, fs, panic};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use object::{Object, ObjectSection, ObjectSymbol};

use rriscv::bus::{Bus, RAM_ADDR};
use rriscv::hart::Hart;
use rriscv::ram::Ram;
use rriscv::rom::Rom;
use rriscv::rtc::Rtc;

fn main() {
    let args: Vec<String> = env::args().collect();
    let elf_file = args.get(1).expect("expect elf file");
    let sig_file = args.get(2);

    let text = fs::read(elf_file).expect("no .text");

    let rom = Rom::new(text);
    let ram = Ram::new();

    let (ram, b, e) = load_elf(elf_file, ram).expect("elf");

    let rtc = Rtc::new();

    let bus = Arc::new(Bus::new(rom, ram, rtc));

    let result = panic::catch_unwind(|| {
        let mut m = Hart::new(0, RAM_ADDR as u32, bus.clone());
        for i in 0..10000 {
            if !m.tick() {
                eprintln!("exited at: {}", i);
                break;
            }
        }
    });
    match result {
        Ok(_) => println!("was ok"),
        Err(_) => println!("was not ok"),
    }

    if let Some(sig_file) = sig_file {
        write_signature(sig_file, bus.clone(), b, e);
    }
}

fn load_elf(elf_file: &String, ram: Ram) -> Result<(Ram, usize, usize), Box<dyn Error>> {
    let bin_data = fs::read(elf_file)?;
    let obj_file = object::File::parse(&*bin_data)?;

    if let Some(section) = obj_file.section_by_name(".text.init") {
        ram.write(
            section.address() as usize - RAM_ADDR,
            section.data()?.to_vec(),
        );
    }
    if let Some(section) = obj_file.section_by_name(".data") {
        ram.write(
            section.address() as usize - RAM_ADDR,
            section.data()?.to_vec(),
        );
    }

    let mut begin_signature: usize = 0;
    let mut end_signature: usize = 0;

    for symbol in obj_file.symbols() {
        match symbol.name() {
            Ok("begin_signature") => begin_signature = symbol.address() as usize,
            Ok("end_signature") => end_signature = symbol.address() as usize,
            _ => {}
        }
    }

    Ok((ram, begin_signature, end_signature))
}

fn write_signature(sig_file: &String, bus: Arc<Bus>, begin: usize, end: usize) {
    //println!("sig betwen {:x}->{:x}", begin, end);

    let mut f = File::create(sig_file).expect("sigfile open");

    let mut i = 1u32;
    for addr in begin..=end {
        let byte = bus.read_byte(addr).expect("ram");
        f.write(format!("{:02x}", byte).as_bytes())
            .expect("writing sig");

        if i > 0 && i % 4 == 0 {
            //println!("{}->{:x}", i, addr);
            f.write("\n".as_bytes()).expect("writing sig");
        }
        i += 1;
    }
}
