#[derive(Debug)]
pub enum Fault {
    MemoryFault(usize),
    Halt,
}
