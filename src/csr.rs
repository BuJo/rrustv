use std::ops::Index;
use std::ops::IndexMut;

pub const NUM_CSRS: usize = 4096;
pub const MSTATUS: usize = 0x300;
pub const MISA: usize = 0x301;
pub const MEDELEG: usize = 0x301;
pub const MVENDORID: usize = 0xF11;
pub const MARCHID: usize = 0xF12;
pub const MIMPID: usize = 0xF13;
pub const MHARTID: usize = 0xF14;

pub const MCYCLE: usize = 0xB00;
pub const MINSTRET: usize = 0xB02;

pub struct Csr {
    csrs: [u32; NUM_CSRS],
}

impl Csr {
    pub fn new() -> Csr {
        Self {
            csrs: [0; NUM_CSRS],
        }
    }
}

impl Index<usize> for Csr {
    type Output = u32;

    fn index(&self, csr: usize) -> &Self::Output {
        &self.csrs[csr]
    }
}
impl IndexMut<usize> for Csr {
    fn index_mut(&mut self, csr: usize) -> &mut Self::Output {
        &mut self.csrs[csr]
    }
}
