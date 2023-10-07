use std::{env, fs};
use std::fs::File;
use std::io::Write;
use std::ops::Range;
use std::sync::Arc;

use object::{Object, ObjectSection, ObjectSymbol};

use rriscv::device::Device;
use rriscv::dynbus::DynBus;
use rriscv::hart::Hart;
use rriscv::htif::Htif;
use rriscv::ram::Ram;
use rriscv::rom::Rom;

fn main() {
    let args: Vec<String> = env::args().collect();
    let elf_file = args.get(1).expect("expect elf file");
    let sig_file = args.get(2);

    let mut bus = DynBus::new();
    let mut pc: usize = 0;

    let bin_data = fs::read(elf_file).expect("file");
    let elf = object::File::parse(&*bin_data).expect("parsing");
    if let Some(section) = elf.section_by_name(".text.init") {
        let start = section.address() as usize;
        let end = start + section.size() as usize;
        let rom = Rom::new(section.data().expect("data").to_vec());
        bus.map(rom, Range { start, end });
        pc = start;
    }

    if let Some(section) = elf.section_by_name(".data") {
        let start = section.address() as usize;
        let end = start + section.size() as usize;
        let ram = Ram::new();
        ram.write(0, section.data().expect("data").to_vec());
        bus.map(ram, Range { start, end });
    }

    if let Some(section) = elf.section_by_name(".tohost") {
        let start = section.address() as usize;
        let end = start + section.size() as usize;
        let htif = Htif::new();
        bus.map(htif, Range { start, end });
    }


    let bus = Arc::new(bus);

    let mut m = Hart::new(0, pc as u32, bus.clone());
    for i in 0..10000 {
        match m.tick() {
            Ok(_) => {}
            Err(e) => {
                eprintln!("exited at: {} ({:?})", i, e);
                break;
            }
        }
    }

    if let Some(sig_file) = sig_file {
        write_signature(sig_file, bus.clone(), elf);
    }
}

fn write_signature(sig_file: &String, bus: Arc<DynBus>, elf: object::File<>) {
    let mut f = File::create(sig_file).expect("sigfile open");

    let mut begin_signature = 0;
    let mut end_signature = 0;
    for symbol in elf.symbols() {
        match symbol.name() {
            Ok("begin_signature") => begin_signature = symbol.address() as usize,
            Ok("end_signature") => end_signature = symbol.address() as usize,
            _ => {}
        }
    }

    let mut i = 1u32;
    for addr in begin_signature..end_signature {
        let byte = bus.read_byte(addr).expect("ram");
        f.write_all(format!("{:02x}", byte).as_bytes())
            .expect("writing sig");

        if i > 0 && i % 4 == 0 {
            f.write_all("\n".as_bytes()).expect("writing sig");
        }
        i += 1;
    }
}
