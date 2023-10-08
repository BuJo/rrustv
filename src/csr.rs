use std::ops::Index;
use std::ops::IndexMut;

const XLEN: u32 = 32;

pub const NUM_CSRS: usize = 4096;

// M-mode registers
pub const MSTATUS: usize = 0x300;
pub const MISA: usize = 0x301;
pub const MEDELEG: usize = 0x301;
pub const MTVEC: usize = 0x305;
pub const MSCRATCH: usize = 0x340;
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
    pub fn new(id: u32) -> Csr {
        let mut csr = Self {
            csrs: [0; NUM_CSRS],
        };

        // RV32 I
        csr[MISA] = 0b01 << (XLEN - 2) | 1 << 8;

        // Non-commercial implementation
        csr[MVENDORID] = 0;

        // Open-Source project, unregistered
        csr[MARCHID] = 0;

        // Version
        csr[MIMPID] = 1;

        // Current hart
        csr[MHARTID] = id;

        // Status
        csr[MEDELEG] = 0;
        csr[MSTATUS] = 0;

        // Cycle counters
        csr[MCYCLE] = 0; // actually per core, not hart
        csr[MINSTRET] = 0;

        csr
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


const NAME_MAP: [(usize, &'static str); 2] = [
    (MSCRATCH, "mscratch"),
    (MTVEC, "mtvec"),
];


impl Csr {
    pub fn name(id: u32) -> &'static str {
        for (i, s) in NAME_MAP {
            if i == (id as usize) {
                return s;
            }
        }
        "U"
    }
}
