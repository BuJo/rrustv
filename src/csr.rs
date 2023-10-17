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

type CsrFn = for<'a> fn(&'a Csr, usize) -> &u32;
type CsrWrFn = for<'a> fn(&'a mut Csr, usize) -> &'a mut u32;

const CSR_MAP: [(usize, &str, CsrFn, CsrWrFn); 99] = [
    // Unprivileged Floating Point
    (0x001, "fflags", handle_nop, handle_nop_wr),
    (0x002, "frm", handle_nop, handle_nop_wr),
    (0x003, "fcsr", handle_nop, handle_nop_wr),
    // Unprivileged Counter/Timers
    (0xC00, "cycle", handle_nop, handle_nop_wr),
    (0xC01, "time", handle_nop, handle_nop_wr),
    (0xC02, "instret", handle_nop, handle_nop_wr),
    (0xC03, "hpmcounter3", handle_nop, handle_nop_wr),
    (0xC04, "hpmcounter4", handle_nop, handle_nop_wr),
    //...
    (0xC1F, "hpmcounter31", handle_nop, handle_nop_wr),
    (0xC80, "cycleeh", handle_nop, handle_nop_wr),
    (0xC81, "intreth", handle_nop, handle_nop_wr),
    (0xC82, "hpmcounter3h", handle_nop, handle_nop_wr),
    (0xC83, "hpmcounter4h", handle_nop, handle_nop_wr),
    //...
    (0xC9F, "hpmcounter31h", handle_nop, handle_nop_wr),
    // Supervisor Trap Setup
    (0x100, "sstatus", handle_nop, handle_nop_wr),
    (0x104, "sie", handle_nop, handle_nop_wr),
    (0x105, "stvec", handle_nop, handle_nop_wr),
    (0x106, "scounteren", handle_nop, handle_nop_wr),
    // Supervisor Configuration
    (0x10A, "sevncfg", handle_nop, handle_nop_wr),
    // Supervisor Trap Handling
    (0x140, "sscratch", handle_nop, handle_nop_wr),
    (0x141, "sepc", handle_nop, handle_nop_wr),
    (0x142, "scause", handle_nop, handle_nop_wr),
    (0x143, "stval", handle_nop, handle_nop_wr),
    (0x144, "sip", handle_nop, handle_nop_wr),
    // Supervisor Protection and Translation
    (0x180, "satp", handle_nop, handle_nop_wr),
    // Supervisor Debug/Trace Registers
    (0x5A8, "scontext", handle_nop, handle_nop_wr),
    // Hypervisor Trap Setup
    (0x600, "hstatus", handle_nop, handle_nop_wr),
    (0x602, "hedeleg", handle_nop, handle_nop_wr),
    (0x603, "hideleg", handle_nop, handle_nop_wr),
    (0x604, "hie", handle_nop, handle_nop_wr),
    (0x606, "hcounteren", handle_nop, handle_nop_wr),
    (0x607, "hgeie", handle_nop, handle_nop_wr),
    // Hypervisor Trap Handling
    (0x643, "htval", handle_nop, handle_nop_wr),
    (0x644, "hip", handle_nop, handle_nop_wr),
    (0x645, "hvpi", handle_nop, handle_nop_wr),
    (0x64A, "htinst", handle_nop, handle_nop_wr),
    (0x6E12, "hgeip", handle_nop, handle_nop_wr),
    // Hypervisor Configuration
    (0x60A, "henvcfg", handle_nop, handle_nop_wr),
    (0x61A, "henvcfgh", handle_nop, handle_nop_wr),
    // Hypervisor Protection and Translation
    (0x680, "hgatp", handle_nop, handle_nop_wr),
    // Hypervisor Debug/Trace Registers
    (0x6A8, "hcontext", handle_nop, handle_nop_wr),
    // Hypervisor Counter/Timer Virtualization Registers
    (0x605, "htimedelta", handle_nop, handle_nop_wr),
    (0x615, "htimedeltah", handle_nop, handle_nop_wr),
    // Hypervisor Virtual Supervisor Registers
    (0x200, "vsstatus", handle_nop, handle_nop_wr),
    (0x204, "vsie", handle_nop, handle_nop_wr),
    (0x205, "vstvec", handle_nop, handle_nop_wr),
    (0x240, "vsscratch", handle_nop, handle_nop_wr),
    (0x241, "vsepc", handle_nop, handle_nop_wr),
    (0x242, "vscause", handle_nop, handle_nop_wr),
    (0x243, "vstval", handle_nop, handle_nop_wr),
    (0x244, "vsip", handle_nop, handle_nop_wr),
    (0x280, "vsatp", handle_nop, handle_nop_wr),
    // Machine Information Reigsers
    (MVENDORID, "mvendorid", Csr::index, Csr::index_mut),
    (MARCHID, "marchid", Csr::index, Csr::index_mut),
    (MIMPID, "mimpid", Csr::index, Csr::index_mut),
    (MHARTID, "mhartid", Csr::index, Csr::index_mut),
    (0xF15, "mconfigptr", Csr::index, Csr::index_mut),
    // Machine Trap Setup
    (MSTATUS, "mstatus", Csr::index, Csr::index_mut),
    (MISA, "misa", Csr::index, Csr::index_mut),
    (MEDELEG, "medeleg", Csr::index, Csr::index_mut),
    (0x303, "mideleg", Csr::index, Csr::index_mut),
    (0x304, "mie", Csr::index, Csr::index_mut),
    (MTVEC, "mtvec", Csr::index, Csr::index_mut),
    (0x306, "mcounteren", Csr::index, Csr::index_mut),
    (0x310, "mstatush", Csr::index, Csr::index_mut),
    // Machine Trap Handling
    (MSCRATCH, "mscratch", Csr::index, Csr::index_mut),
    (0x341, "mepc", Csr::index, Csr::index_mut),
    (0x342, "mcause", Csr::index, Csr::index_mut),
    (0x343, "mtval", Csr::index, Csr::index_mut),
    (0x344, "mip", Csr::index, Csr::index_mut),
    (0x34A, "minst", Csr::index, Csr::index_mut),
    (0x34B, "mtval2", Csr::index, Csr::index_mut),
    // Machine Configuration
    (0x30A, "menvcfg", Csr::index, Csr::index_mut),
    (0x31A, "menvcfgh", Csr::index, Csr::index_mut),
    (0x347, "mseccfg", Csr::index, Csr::index_mut),
    (0x357, "mseccfgh", Csr::index, Csr::index_mut),
    // Machine Memory Protection
    (0x3A0, "pmpcfg0", Csr::index, Csr::index_mut),
    //...
    (0x3AF, "pmpaddr0", Csr::index, Csr::index_mut),
    (0x3EF, "pmpaddr63", Csr::index, Csr::index_mut),
    // Machine Counters/Timers
    (MCYCLE, "mcycle", Csr::index, Csr::index_mut),
    (MINSTRET, "minstret", Csr::index, Csr::index_mut),
    (0xB03, "mhpmcounter3", Csr::index, Csr::index_mut),
    (0xB1F, "mhpmcounter31", Csr::index, Csr::index_mut),
    (0xB80, "mcycleh", Csr::index, Csr::index_mut),
    (0xB82, "minstreth", Csr::index, Csr::index_mut),
    (0xB82, "mhpmcounter3h", Csr::index, Csr::index_mut),
    (0xB9F, "mhpmcounter31h", Csr::index, Csr::index_mut),
    // Machine Counter Setup
    (0x320, "mcountinhibit", Csr::index, Csr::index_mut),
    (0x323, "mhpmevent3", Csr::index, Csr::index_mut),
    (0x33F, "mhpmevent31", Csr::index, Csr::index_mut),
    // Machine Debug/Trace Registers (Shared with Debug Mode)
    (0x7A0, "tselect", Csr::index, Csr::index_mut),
    (0x7A1, "tdata1", Csr::index, Csr::index_mut),
    (0x7A2, "tdata2", Csr::index, Csr::index_mut),
    (0x7A3, "tdata3", Csr::index, Csr::index_mut),
    (0x7A4, "mcontext", Csr::index, Csr::index_mut),
    // Machine Debug Mode Registers
    (0x7B0, "dcsr", Csr::index, Csr::index_mut),
    (0x7B1, "dpc", Csr::index, Csr::index_mut),
    (0x7B2, "dscratch0", Csr::index, Csr::index_mut),
    (0x7B3, "dscratch1", Csr::index, Csr::index_mut),
];

fn handle_nop_wr(csr: &mut Csr, _num: usize) -> &mut u32 {
    csr.index_mut(MSCRATCH)
}

fn handle_nop(csr: &Csr, _num: usize) -> &u32 {
    csr.index(MSCRATCH)
}

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

impl Csr {
    pub fn name(id: u32) -> &'static str {
        for (i, s, ..) in CSR_MAP {
            if i == (id as usize) {
                return s;
            }
        }
        "U"
    }
}
