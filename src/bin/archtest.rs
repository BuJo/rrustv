use rriscv::bus::Bus;
use rriscv::hart::Hart;
use rriscv::ram::Ram;
use rriscv::rom::Rom;
use rriscv::rtc::Rtc;
use std::error::Error;
use std::sync::Arc;
use std::{env, fs};
use object::{Object, ObjectSection};

fn main() {
    let args: Vec<String> = env::args().collect();
    let elf_file = args.get(1).expect("expect elf file");
    let _sig_file = args.get(1).expect("expect sig file");

    let text = fs::read(elf_file).expect("no .text");

    let rom = Rom::new(text);
    let ram = Ram::new();

    let ram = load_elf(elf_file, ram).expect("elf");

    let rtc = Rtc::new();

    let bus = Arc::new(Bus::new(rom, ram, rtc));

    let mut m = Hart::new(0, bus);
    m.pc = 0x80000000;
    for i in 0..10000 {
        if !m.tick() {
            eprintln!("exited at: {}", i);
            break;
        }
    }
}

fn load_elf(elf_file: &String, ram: Ram) -> Result<Ram, Box<dyn Error>> {
    let bin_data = fs::read(elf_file)?;
    let obj_file = object::File::parse(&*bin_data)?;


    if let Some(section) = obj_file.section_by_name(".text.init") {
        ram.write((section.address() - 0x80000000) as usize, section.data()?.to_vec());
    }
    if let Some(section) = obj_file.section_by_name(".tohost") {
        ram.write((section.address() - 0x80000000) as usize, section.data()?.to_vec());
    }
    if let Some(section) = obj_file.section_by_name(".data") {
        ram.write((section.address() - 0x80000000) as usize, section.data()?.to_vec());
    }

    Ok(ram)
}
