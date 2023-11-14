use std::sync::Arc;
use std::{env, fs};
use std::ops::Range;
use object::{Object, ObjectSection};

use rriscv::dynbus::DynBus;
use rriscv::hart::Hart;
use rriscv::ram::Ram;
use rriscv::reg::treg;

fn main() {
    let args: Vec<String> = env::args().collect();
    let image_file = args.get(1).expect("expect image file");
    let bin_data = fs::read(image_file).expect("file");

    let mut emu = Emulator::new(bin_data);
    emu.run()
}

struct Emulator  {
    hart: Hart<DynBus>
}

impl Emulator {
    fn new(bin_data: Vec<u8>) -> Emulator {

        let mut bus = DynBus::new();
        let elf = object::File::parse(&*bin_data).expect("parsing");

        let section = elf.section_by_name(".text").expect("need text section");
        let start = section.address() as usize;
        let pc = start;

        let ram = Ram::new();
        ram.write(0, section.data().expect("data").to_vec());
        bus.map(ram, Range { start: pc, end: pc+0x100000 });

        let bus = Arc::new(bus);

        let mut hart = Hart::new(0, pc, bus.clone());

        hart.set_register(treg("sp"), (pc+0x100000) as u64);

        Self {
            hart
        }
    }

    fn run(&mut self) {
        let mut i = 0;

        loop {
            match self.hart.tick() {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("exited at: {} ({:?})", i, e);
                    break;
                }
            }
            i += 1;
        }
    }
}
