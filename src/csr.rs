const XLEN: u64 = 32;

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
pub const SATP: usize = 0x180;

type CsrFn = for<'a> fn(&'a Csr, usize) -> u64;
type CsrWrFn = for<'a> fn(&'a mut Csr, usize, u64);

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
    (SATP, "satp", handle_nop, handle_nop_wr),
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
    (MVENDORID, "mvendorid", Csr::read_any, Csr::write_any),
    (MARCHID, "marchid", Csr::read_any, Csr::write_any),
    (MIMPID, "mimpid", Csr::read_any, Csr::write_any),
    (MHARTID, "mhartid", Csr::read_any, Csr::write_any),
    (0xF15, "mconfigptr", Csr::read_any, Csr::write_any),
    // Machine Trap Setup
    (MSTATUS, "mstatus", Csr::read_any, Csr::write_any),
    (MISA, "misa", Csr::read_any, Csr::write_any),
    (MEDELEG, "medeleg", Csr::read_any, Csr::write_any),
    (0x303, "mideleg", Csr::read_any, Csr::write_any),
    (0x304, "mie", Csr::read_any, Csr::write_any),
    (MTVEC, "mtvec", Csr::read_mtvec, Csr::write_any),
    (0x306, "mcounteren", Csr::read_any, Csr::write_any),
    (0x310, "mstatush", Csr::read_any, Csr::write_any),
    // Machine Trap Handling
    (MSCRATCH, "mscratch", Csr::read_any, Csr::write_any),
    (0x341, "mepc", Csr::read_any, Csr::write_any),
    (0x342, "mcause", Csr::read_any, Csr::write_any),
    (0x343, "mtval", Csr::read_any, Csr::write_any),
    (0x344, "mip", Csr::read_any, Csr::write_any),
    (0x34A, "minst", Csr::read_any, Csr::write_any),
    (0x34B, "mtval2", Csr::read_any, Csr::write_any),
    // Machine Configuration
    (0x30A, "menvcfg", Csr::read_any, Csr::write_any),
    (0x31A, "menvcfgh", Csr::read_any, Csr::write_any),
    (0x347, "mseccfg", Csr::read_any, Csr::write_any),
    (0x357, "mseccfgh", Csr::read_any, Csr::write_any),
    // Machine Memory Protection
    (0x3A0, "pmpcfg0", Csr::read_any, Csr::write_any),
    //...
    (0x3AF, "pmpaddr0", Csr::read_any, Csr::write_any),
    (0x3EF, "pmpaddr63", Csr::read_any, Csr::write_any),
    // Machine Counters/Timers
    (MCYCLE, "mcycle", Csr::read_any, Csr::write_any),
    (MINSTRET, "minstret", Csr::read_any, Csr::write_any),
    (0xB03, "mhpmcounter3", Csr::read_any, Csr::write_any),
    (0xB1F, "mhpmcounter31", Csr::read_any, Csr::write_any),
    (0xB80, "mcycleh", Csr::read_any, Csr::write_any),
    (0xB82, "minstreth", Csr::read_any, Csr::write_any),
    (0xB82, "mhpmcounter3h", Csr::read_any, Csr::write_any),
    (0xB9F, "mhpmcounter31h", Csr::read_any, Csr::write_any),
    // Machine Counter Setup
    (0x320, "mcountinhibit", Csr::read_any, Csr::write_any),
    (0x323, "mhpmevent3", Csr::read_any, Csr::write_any),
    (0x33F, "mhpmevent31", Csr::read_any, Csr::write_any),
    // Machine Debug/Trace Registers (Shared with Debug Mode)
    (0x7A0, "tselect", Csr::read_any, Csr::write_any),
    (0x7A1, "tdata1", Csr::read_any, Csr::write_any),
    (0x7A2, "tdata2", Csr::read_any, Csr::write_any),
    (0x7A3, "tdata3", Csr::read_any, Csr::write_any),
    (0x7A4, "mcontext", Csr::read_any, Csr::write_any),
    // Machine Debug Mode Registers
    (0x7B0, "dcsr", Csr::read_any, Csr::write_any),
    (0x7B1, "dpc", Csr::read_any, Csr::write_any),
    (0x7B2, "dscratch0", Csr::read_any, Csr::write_any),
    (0x7B3, "dscratch1", Csr::read_any, Csr::write_any),
];

fn handle_nop_wr(_csr: &mut Csr, _num: usize, _val: u64) {
    // ignore
}

fn handle_nop(_csr: &Csr, _num: usize) -> u64 {
    // ignore
    0
}

pub struct Csr {
    csrs: [u64; NUM_CSRS],
}

impl Csr {
    pub fn new(id: u64) -> Csr {
        let mut csr = Self {
            csrs: [0; NUM_CSRS],
        };

        // RV32 I
        csr.csrs[MISA] = 0b01 << (XLEN - 2) | 1 << 8;

        // Non-commercial implementation
        csr.csrs[MVENDORID] = 0;

        // Open-Source project, unregistered
        csr.csrs[MARCHID] = 0;

        // Version
        csr.csrs[MIMPID] = 1;

        // Current hart
        csr.csrs[MHARTID] = id;

        // Status
        csr.csrs[MEDELEG] = 0;
        csr.csrs[MSTATUS] = 0;

        // Cycle counters
        csr.csrs[MCYCLE] = 0; // actually per core, not hart
        csr.csrs[MINSTRET] = 0;

        csr
    }
}

impl Csr {
    pub fn name(csr: usize) -> &'static str {
        for (i, s, ..) in CSR_MAP {
            if i == csr {
                return s;
            }
        }
        "U"
    }

    pub(crate) fn read(&self, csr: usize) -> u64 {
        eprintln!("r csr {}[{:x}]", Csr::name(csr), self.csrs[csr]);

        for (i, _s, r, _w) in CSR_MAP {
            if i == csr {
                return r(self, csr);
            }
        }

        0
    }

    pub(crate) fn write(&mut self, csr: usize, val: u64) {
        eprintln!(
            "w csr {}[{:x}]->[{:x}]",
            Csr::name(csr),
            self.csrs[csr],
            val
        );

        for (i, _s, _r, w) in CSR_MAP {
            if i == csr {
                return w(self, csr, val);
            }
        }
    }

    fn read_any(&self, csr: usize) -> u64 {
        self.csrs[csr]
    }

    fn write_any(&mut self, csr: usize, val: u64) {
        self.csrs[csr] = val
    }

    // WARL
    fn read_mtvec(&self, csr: usize) -> u64 {
        let val = &self.csrs[csr];
        let base = val >> 2;
        let mode = val & 0b11;

        // legality: mode >= 2 is reserved
        let mode = mode & 0b01;

        // legality: base must be aligned to 4 byte boundary
        let base = (base >> 2) << 2;

        let legal_val = (base << 2) | mode;

        eprintln!(
            "r csr {}[{:x}]->[{:x}]",
            Csr::name(csr),
            self.csrs[csr],
            legal_val
        );

        legal_val
    }
}
