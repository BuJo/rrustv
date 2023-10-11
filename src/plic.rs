#[derive(Debug)]
pub enum Fault {
    MemoryFault(usize),
    Unaligned(usize),
    Halt,
    Unimplemented,
    IllegalOpcode(u32),
}
