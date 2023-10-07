use std::ops::Range;

pub static MAPPED_AT: Range<usize> = 0x4000..0x5000;
static HALT: usize = 0xfd40 - MAPPED_AT.start;

pub struct Htif {}

impl Htif {
    pub fn new() -> Htif {
        Htif {}
    }
}
