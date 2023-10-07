use std::fmt;
use std::sync::Arc;

use InstructionFormat::{B, I, J, R, S, U};

use crate::csr;
use crate::csr::Csr;
use crate::device::Device;
use crate::plic::Fault;
use crate::plic::Fault::{Halt, Unimplemented};
use crate::see;

pub struct Hart<BT: Device> {
    start_pc: u32,

    bus: Arc<BT>,
    registers: [u32; 32],
    pc: u32,
    csr: Csr,

    stop: bool,
}

impl<BT: Device> Hart<BT> {
    pub fn new(id: u32, pc: u32, bus: Arc<BT>) -> Self {
        let mut m = Hart {
            start_pc: pc,
            bus,
            registers: [0; 32],
            pc,
            csr: Csr::new(id),
            stop: false,
        };

        m.reset();

        m
    }

    pub fn reset(&mut self) {
        self.pc = self.start_pc;
        self.registers = [0; 32];
    }

    pub fn stop(&mut self) {
        self.stop = true;
    }

    pub fn tick(&mut self) -> Result<(), Fault> {
        if self.stop {
            return Err(Halt);
        }

        let res = self
            .fetch_instruction()
            .and_then(|instruction| self.decode_instruction(instruction))
            .and_then(|(ins, decoded)| self.execute_instruction(decoded, ins));

        // simulate passing of time
        self.csr[csr::MCYCLE] += 3;
        self.csr[csr::MINSTRET] += 1;

        res
    }

    pub fn set_register(&mut self, reg: u8, val: u32) {
        match reg {
            0 => {}
            1..=31 => self.registers[reg as usize] = val,
            _ => panic!(),
        }
    }

    pub fn get_register(&self, reg: u8) -> u32 {
        match reg {
            0..=31 => self.registers[reg as usize],
            _ => panic!(),
        }
    }

    fn fetch_instruction(&mut self) -> Result<u32, Fault> {
        let ins = self.bus.read_word(self.pc as usize);
        self.pc += 4;
        ins
    }

    fn decode_instruction(&self, instruction: u32) -> Result<(u32, InstructionFormat), Fault> {
        let opcode = (instruction & 0b1111111) as u8;
        let decoded = match opcode {
            0b0110011 => {
                let rd = ((instruction >> 7) & 0b11111) as u8;
                let funct3 = ((instruction >> 12) & 0b111) as u8;
                let rs1 = ((instruction >> 15) & 0b11111) as u8;
                let rs2 = ((instruction >> 20) & 0b11111) as u8;
                let funct7 = (instruction >> 25) as u8;
                R {
                    opcode,
                    rd,
                    funct3,
                    rs1,
                    rs2,
                    funct7,
                }
            }
            0b0010011 | 0b0000011 | 0b1100111 | 0b1110011 => {
                let rd = ((instruction & 0x0F80) >> 7) as u8;
                let funct3 = ((instruction & 0x7000) >> 12) as u8;
                let rs1 = ((instruction & 0xF8000) >> 15) as u8;
                let imm = ((instruction & 0xfff00000) as i32 as u64 >> 20) as i16;
                I {
                    opcode,
                    rd,
                    funct3,
                    rs1,
                    imm,
                }
            }
            0b0100011 => {
                let funct3 = ((instruction >> 12) & 0b111) as u8;
                let rs1 = ((instruction >> 15) & 0b11111) as u8;
                let rs2 = ((instruction >> 20) & 0b11111) as u8;
                let imm7 = (instruction >> 7) & 0b11111;
                let imm25 = instruction & 0xfe000000;
                let imm = ((imm25 + (imm7 << 20)) as i32 as u64 >> 20) as i16;
                S {
                    opcode,
                    funct3,
                    rs1,
                    rs2,
                    imm,
                }
            }
            0b1100011 => {
                let funct3 = ((instruction >> 12) & 0b111) as u8;
                let rs1 = ((instruction >> 15) & 0b1111) as u8;
                let rs2 = ((instruction >> 20) & 0b1111) as u8;
                let imm = ((instruction & 0x8000_0000) >> 19)
                    | ((instruction & 0x7e00_0000) >> 20)
                    | ((instruction & 0x0000_0f00) >> 7)
                    | ((instruction & 0x0000_0080) << 4);
                let imm = ((imm << 19) >> 19) as i16;

                B {
                    opcode,
                    funct3,
                    rs1,
                    rs2,
                    imm,
                }
            }
            0b1101111 => {
                let rd = ((instruction & 0x0F80) >> 7) as u8;
                let imm = ((instruction & 0x8000_0000) >> 11)
                    | ((instruction & 0x7fe0_0000) >> 20)
                    | ((instruction & 0x0010_0000) >> 9)
                    | (instruction & 0x000f_f000);
                let imm = ((imm << 11) as i32) >> 11;

                J { opcode, rd, imm }
            }
            0b0110111 | 0b0010111 => {
                let rd = ((instruction >> 7) & 0x1F) as u8;
                let imm = ((instruction & 0xfffff800) as i32 as u64 >> 12) as i32;
                U { opcode, rd, imm }
            }
            _ => {
                eprintln!(
                    "[{}] [{:#x}] {:07b} Unknown opcode for ins {:08x}",
                    self.csr[csr::MHARTID],
                    self.pc,
                    opcode,
                    instruction
                );
                return Err(Unimplemented);
            }
        };

        Ok((instruction, decoded))
    }

    fn execute_instruction(
        &mut self,
        instruction: InstructionFormat,
        ins: u32,
    ) -> Result<(), Fault> {
        match instruction {
            // RV32I

            // ADD
            R {
                opcode: 0b0110011,
                rd,
                funct3: 0x0,
                rs1,
                rs2,
                funct7: 0x00,
            } => {
                let val = self.get_register(rs1).wrapping_add(self.get_register(rs2));
                self.set_register(rd, val);

                self.dbgins(ins, format!("add\t{},{},{}", reg(rd), reg(rs1), reg(rs2)))
            }
            // AND
            R {
                opcode: 0b0110011,
                rd,
                funct3: 0x7,
                rs1,
                rs2,
                funct7: 0x00,
            } => {
                let val = self.get_register(rs1) & self.get_register(rs2);
                self.set_register(rd, val);

                self.dbgins(ins, format!("and\t{},{},{}", reg(rd), reg(rs1), reg(rs2)))
            }
            // ADD immediate
            I {
                opcode: 0b0010011,
                rd,
                funct3: 0x0,
                rs1,
                imm,
            } => {
                let val = self.get_register(rs1).wrapping_add(imm as u32);
                self.set_register(rd, val);

                self.dbgins(
                    ins,
                    format!("add\t{},{},{} # {:x}", reg(rd), reg(rs1), imm, val),
                )
            }
            // lb Load Byte
            I {
                opcode: 0b0000011,
                rd,
                funct3: 0x0,
                rs1,
                imm,
            } => {
                let addr = (self.get_register(rs1).wrapping_add(imm as u32)) as usize;
                let val = self.bus.read_byte(addr).expect("address being readable");
                self.set_register(rd, val as u32);

                self.dbgins(ins, format!("lb\t{},{},{:#x}", reg(rd), reg(rs1), imm))
            }
            // sb Store Byte
            S {
                opcode: 0b0100011,
                funct3: 0x0,
                rs1,
                rs2,
                imm,
            } => {
                let addr = (self.get_register(rs1).wrapping_add(imm as u32)) as usize;
                let val = self.get_register(rs2 & 0xF) as u8;
                self.bus
                    .write_byte(addr, val)
                    .expect("address being writeable");

                self.dbgins(ins, format!("sb {},{},{:#x}", reg(rs1), reg(rs2), imm))
            }
            // sw Store Word
            S {
                opcode: 0b0100011,
                funct3: 0x2,
                rs1,
                rs2,
                imm,
            } => {
                let addr = (self.get_register(rs1).wrapping_add(imm as u32)) as usize;
                let val = self.get_register(rs2);
                self.dbgins(ins, format!("sw\t{},{}({})", reg(rs2), imm, reg(rs1)));
                return self.bus.write_word(addr, val);
            }
            // beq Branch ==
            B {
                opcode: 0b1100011,
                funct3: 0x00,
                rs1,
                rs2,
                imm,
            } => {
                if self.get_register(rs1) == self.get_register(rs2) {
                    if imm > 0 {
                        // increment program counter, without the current address
                        self.pc = self.pc.wrapping_add(imm as u32) - 4
                    } else {
                        // decrement program counter, without current address and disregarding rmb
                        self.pc = self.pc.wrapping_add(imm as u32) - 1 - 4
                    }
                }
                self.dbgins(ins, format!("beq\t{},{},{}", reg(rs1), reg(rs2), imm))
            }
            // jal Jump And Link
            J {
                opcode: 0b1101111,
                rd,
                imm,
            } => {
                self.set_register(rd, self.pc);
                self.pc = self.pc.wrapping_add(imm as u32);

                self.dbgins(ins, format!("jal\t{},{:#x}", reg(rd), imm))
            }

            // lui Load Upper Imm
            U {
                opcode: 0b0110111,
                rd,
                imm,
            } => {
                // one instruction length less
                let val = (imm as u32) << 12;
                self.set_register(rd, val);

                self.dbgins(ins, format!("lui\t{},{:#x}", reg(rd), imm))
            }
            // auipc Add Upper Imm to PC
            U {
                opcode: 0b0010111,
                rd,
                imm,
            } => {
                // one instruction length less
                let val = (self.pc - 4) + ((imm as u32) << 12);
                self.set_register(rd, val);

                self.dbgins(ins, format!("auipc\t{},{:#x}", reg(rd), imm))
            }

            // ecall Environment Call
            I {
                opcode: 0b1110011,
                funct3: 0x0,
                imm: 0x0,
                ..
            } => {
                // We're unprivileged machine mode, no need to check SEDELEG
                see::call(self);

                self.dbgins(ins, "ecall".to_string())
            }
            // ebreak Environment Break
            I {
                opcode: 0b1110011,
                funct3: 0x0,
                imm: 0x1,
                ..
            } => {
                // Stop the hart, the Execution Environment has to take over
                self.stop = true;

                self.dbgins(ins, "ebreak".to_string())
            }
            _ => {
                eprintln!(
                    "[{}] Unknown instruction: {:}",
                    self.csr[csr::MHARTID],
                    instruction
                );
                return Err(Fault::MemoryFault(self.pc as usize));
            }
        };
        Ok(())
    }

    fn dbgins(&self, ins: u32, asm: String) {
        //eprintln!("[{}] {:}: {}", self.csr[csr::MHARTID], ins, asm);
        eprintln!("{:08x}:\t{:08x}          \t{}", self.pc - 4, ins, asm)
    }
}

const REGMAP: [(u8, &str); 32] = [
    (0, "zero"),
    (1, "ra"),
    (2, "sp"),
    (3, "gp"),
    (4, "tp"),
    (5, "t0"),
    (6, "t1"),
    (7, "t2"),
    (8, "s0"),
    (9, "s1"),
    (10, "a0"),
    (11, "a1"),
    (12, "a2"),
    (13, "a3"),
    (14, "a4"),
    (15, "a5"),
    (16, "a6"),
    (17, "a7"),
    (18, "s2"),
    (19, "s3"),
    (20, "s4"),
    (21, "s5"),
    (22, "s6"),
    (23, "s7"),
    (24, "s8"),
    (25, "s9"),
    (26, "s10"),
    (27, "s11"),
    (28, "t3"),
    (29, "t4"),
    (30, "t5"),
    (31, "t6"),
];

fn reg(reg: u8) -> &'static str {
    for (i, s) in REGMAP {
        if i == reg {
            return s;
        }
    }
    "U"
}

#[derive(Debug)]
pub enum InstructionFormat {
    R {
        opcode: u8,
        rd: u8,
        funct3: u8,
        rs1: u8,
        rs2: u8,
        funct7: u8,
    },
    I {
        opcode: u8,
        rd: u8,
        funct3: u8,
        rs1: u8,
        imm: i16,
    },
    S {
        opcode: u8,
        funct3: u8,
        rs1: u8,
        rs2: u8,
        imm: i16,
    },
    B {
        opcode: u8,
        funct3: u8,
        rs1: u8,
        rs2: u8,
        imm: i16,
    },
    U {
        opcode: u8,
        rd: u8,
        imm: i32,
    },
    J {
        opcode: u8,
        rd: u8,
        imm: i32,
    },
}

impl fmt::Display for InstructionFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            R {
                opcode,
                rd,
                funct3,
                rs1,
                rs2,
                funct7,
            } => {
                write!(
                    f,
                    "R 0b{:07b} 0x{:0x} 0x{:02x} 0x{:02x} ← 0x{:02x} · 0x{:02x}",
                    opcode, funct3, funct7, rd, rs1, rs2
                )
            }
            I {
                opcode,
                rd,
                funct3,
                rs1,
                imm,
            } => {
                write!(
                    f,
                    "I 0b{:07b} 0x{:0x} 0x{:02x} ← 0x{:02x} · {}",
                    opcode, funct3, rd, rs1, imm
                )
            }
            S {
                opcode,
                funct3,
                rs1,
                rs2,
                imm,
            } => {
                write!(
                    f,
                    "S 0b{:07b} 0x{:0x} M[0x{:02x}+{}] ← 0x{:02x}",
                    opcode, funct3, rs1, imm, rs2
                )
            }
            B {
                opcode,
                funct3,
                rs1,
                rs2,
                imm,
            } => {
                write!(
                    f,
                    "B 0b{:07b} 0x{:0x} 0x{:02x} · 0x{:02x} → {}",
                    opcode, funct3, rs1, rs2, imm
                )
            }
            U { opcode, rd, imm } => {
                write!(f, "U 0b{:07b} 0x{:02x} ← {}", opcode, rd, imm)
            }
            J { opcode, rd, imm } => {
                write!(f, "J 0b{:07b} 0x{:02x} ← {}", opcode, rd, imm)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::bus::Bus;
    use crate::hart::{Hart, InstructionFormat, REGMAP};
    use crate::ram::Ram;
    use crate::rom::Rom;

    fn treg(reg: &str) -> u8 {
        for (i, s) in REGMAP {
            if s == reg {
                return i;
            }
        }
        255
    }

    #[test]
    fn addi() {
        let rom = Rom::new(vec![0x13, 0x81, 0x00, 0x7d]);
        let ram = Ram::new();
        let bus = Bus::new(rom, ram);
        let mut m = Hart::new(0, 0, Arc::new(bus));
        m.tick().expect("tick");
        assert_eq!(m.get_register(2), 2000, "x1 mismatch");
    }

    #[test]
    fn addi_neg() {
        let rom = Rom::new(vec![0x93, 0x01, 0x81, 0xc1]);
        let ram = Ram::new();
        let bus = Bus::new(rom, ram);
        let mut m = Hart::new(0, 0, Arc::new(bus));
        m.tick().expect("tick");
        assert_eq!(m.get_register(3) as i32, -1000, "x1 mismatch");
    }

    #[test]
    fn it_works() {
        let rom = Rom::new(vec![
            0x93, 0x00, 0x80, 0x3e, // li	ra,1000
            0x13, 0x81, 0x00, 0x7d, // addi	sp,ra,2000
            0x93, 0x01, 0x81, 0xc1, // addi	gp,sp,-1000
            0x13, 0x82, 0x01, 0x83, // addi	tp,gp,-2000
            0x93, 0x02, 0x82, 0x3e, // addi	t0,tp,1000
            0x13, 0x03, 0x00, 0x04, // li	t1,64
            0x13, 0x03, 0x43, 0x00, // addi	t1,t1,4
        ]);
        let ram = Ram::new();
        let bus = Bus::new(rom, ram);
        let mut m = Hart::new(0, 0, Arc::new(bus));
        for _ in 0..=6 {
            m.tick().expect("tick");
        }
        assert_eq!(m.get_register(0), 0, "zero register must be zero");
        assert_eq!(m.get_register(1), 1000, "x1 mismatch");
        assert_eq!(m.get_register(2), 3000, "x2 mismatch");
        assert_eq!(m.get_register(3), 2000, "x3 mismatch");
        assert_eq!(m.get_register(4), 0, "x4 mismatch");
        assert_eq!(m.get_register(5), 1000, "x5 mismatch");
        assert_eq!(m.get_register(6), 0x40 + 4, "deadbeef");
    }

    fn hart() -> Hart<Bus> {
        let rom = Rom::new(vec![]);
        let ram = Ram::new();
        let bus = Bus::new(rom, ram);
        Hart::new(0, 0, Arc::new(bus))
    }

    #[test]
    fn test_sw_80000130() {
        let ins = 0x0181a023;
        let m = hart();

        let decoded = m.decode_instruction(ins).expect("decode").1;
        match decoded {
            InstructionFormat::S {
                opcode,
                funct3,
                rs1,
                rs2,
                imm,
            } => {
                assert_eq!(opcode, 0b0100011, "opcode wrong");
                assert_eq!(funct3, 0x2, "funct3 wrong");
                assert_eq!(rs1, 3, "rs1 wrong");
                assert_eq!(rs2, 24, "rs2 wrong");
                assert_eq!(imm, 0, "imm wrong");
            }
            _ => assert!(false, "not S"),
        }
    }

    #[test]
    fn test_add_80000154() {
        let ins = 0x015a8ab3;
        let m = hart();

        let decoded = m.decode_instruction(ins).expect("decode").1;
        match decoded {
            InstructionFormat::R {
                opcode,
                funct3,
                funct7,
                rd,
                rs1,
                rs2,
            } => {
                assert_eq!(opcode, 0b0110011, "opcode wrong");
                assert_eq!(funct3, 0x0, "funct3 wrong");
                assert_eq!(funct7, 0x00, "funct7 wrong");
                assert_eq!(rs1, 21, "rs1 wrong");
                assert_eq!(rs2, 21, "rs2 wrong");
                assert_eq!(rd, 21, "rd wrong");
            }
            _ => assert!(false, "not R"),
        }
    }

    #[test]
    fn test_addi_8000015c() {
        let ins = 0xffe00b13;
        let m = hart();

        let decoded = m.decode_instruction(ins).expect("decode").1;
        match decoded {
            InstructionFormat::I {
                opcode,
                funct3,
                rd,
                rs1,
                imm,
            } => {
                assert_eq!(opcode, 0b0010011, "opcode wrong");
                assert_eq!(funct3, 0x0, "funct3 wrong");
                assert_eq!(rd, 22, "rd wrong");
                assert_eq!(rs1, 0, "rs1 wrong");
                assert_eq!(imm, -2, "imm wrong");
            }
            _ => assert!(false, "not I"),
        }
    }

    #[test]
    fn test_lw_800032a0() {
        let ins = 0x17812483;
        let m = hart();

        let decoded = m.decode_instruction(ins).expect("decode").1;
        match decoded {
            InstructionFormat::I {
                opcode,
                funct3,
                rd,
                rs1,
                imm,
            } => {
                assert_eq!(opcode, 0b0000011, "opcode wrong");
                assert_eq!(funct3, 0x2, "funct3 wrong");
                assert_eq!(rd, treg("s1"), "rd wrong");
                assert_eq!(rs1, treg("sp"), "rs1 wrong");
                assert_eq!(imm, 376, "imm wrong");
            }
            _ => assert!(false, "not I"),
        }
    }

    #[test]
    fn test_jal_8000329c() {
        let ins = 0x0200006f;
        let mut m = hart();
        m.pc = 0x8000329c;

        let decoded = m.decode_instruction(ins).expect("decode").1;
        match decoded {
            InstructionFormat::J { opcode, rd, imm } => {
                assert_eq!(opcode, 0b1101111, "opcode wrong");
                assert_eq!(rd, treg("zero"), "rd wrong");
                assert_eq!(imm, 32, "imm wrong");
            }
            _ => assert!(false, "not J"),
        }

        m.execute_instruction(decoded, ins).expect("execute");

        assert_eq!(m.pc, 0x800032bc);
    }

    #[test]
    fn test_auipc_800032c0() {
        let ins = 0x00001f17;
        let mut m = hart();
        m.pc = 0x800032c0 + 4;

        let decoded = m.decode_instruction(ins).expect("decode").1;
        match decoded {
            InstructionFormat::U { opcode, rd, imm } => {
                assert_eq!(opcode, 0b0010111, "opcode wrong");
                assert_eq!(rd, treg("t5"), "rd wrong");
                assert_eq!(imm, 0x1, "imm wrong");
            }
            _ => assert!(false, "not auipc"),
        }

        m.execute_instruction(decoded, ins).expect("execute");

        assert_eq!(m.get_register(treg("t5")), 0x800032c0 + (0x1 << 12));
    }

    #[test]
    fn test_magic_800032c4() {
        let ins = 0xd41f2023;
        let m = hart();

        let decoded = m.decode_instruction(ins).expect("decode").1;
        match decoded {
            InstructionFormat::S {
                opcode,
                funct3,
                rs1,
                rs2,
                imm,
            } => {
                assert_eq!(opcode, 0b0100011, "opcode wrong");
                assert_eq!(funct3, 0x2, "funct3 wrong");
                assert_eq!(rs1, treg("t5"), "rs1 wrong");
                assert_eq!(rs2, treg("ra"), "rs2 wrong");
                assert_eq!(imm, -704, "imm wrong");
            }
            _ => assert!(false, "not sw"),
        }
    }
}
