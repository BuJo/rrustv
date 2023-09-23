use std::ops::Index;
use std::ops::IndexMut;

pub const CSR_SIZE: usize = 4096;
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
    pub csrs: [u32; CSR_SIZE],
}

impl Csr {
    pub fn new() -> Csr {
        Self { csrs: [0; CSR_SIZE] }
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
