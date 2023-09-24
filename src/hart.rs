use std::sync::Arc;
use std::{fmt, process};

use InstructionFormat::{B, I, J, R, S, U};

use crate::csr;
use crate::csr::Csr;
use crate::ram::Ram;
use crate::see;

const XLEN: usize = 32;

pub struct Hart {
    memory: Arc<Ram>,
    registers: [u32; 32],
    pc: u32,
    csr: Csr,

    stop: bool,
}

impl Hart {
    pub(crate) fn new(ram: Arc<Ram>) -> Self {
        let mut m = Hart {
            memory: ram,
            registers: [0; 32],
            pc: 0,
            csr: Csr::new(),
            stop: false,
        };

        // RV32 I
        m.csr[csr::MISA] = 0b01 << XLEN - 2 | 1 << 8;

        // Non-commercial implementation
        m.csr[csr::MVENDORID] = 0;

        // Open-Source project, unregistered
        m.csr[csr::MARCHID] = 0 << XLEN - 1 | 0;

        // Version
        m.csr[csr::MIMPID] = 1;

        // Current hart
        m.csr[csr::MHARTID] = 0;

        m.reset();

        m
    }

    pub(crate) fn reset(&mut self) {
        // Status
        self.csr[csr::MEDELEG] = 0;
        self.csr[csr::MSTATUS] = 0;

        // Cycle counters
        self.csr[csr::MCYCLE] = 0; // actually per core, not hart
        self.csr[csr::MINSTRET] = 0;

        self.pc = 0;
        self.registers = [0; 32];
        self.csr = Csr::new();
    }

    pub(crate) fn stop(&mut self) {
        self.stop = true;
    }

    pub(crate) fn tick(&mut self) -> bool {
        if self.stop {
            return false;
        }

        let instruction = self.fetch_instruction();
        let instruction = self.decode_instruction(instruction);
        self.execute_instruction(instruction);

        // simulate passing of time
        self.csr[csr::MCYCLE] += 3;
        self.csr[csr::MINSTRET] += 1;

        true
    }

    pub fn set_register(&mut self, reg: u8, val: u32) {
        //eprintln!("Setting register {} to 0x{:04x}", reg, val);
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

    fn fetch_instruction(&mut self) -> u32 {
        let ins = self.memory.read_word(self.pc as usize);
        self.pc += 4;
        ins
    }

    fn decode_instruction(&self, instruction: u32) -> InstructionFormat {
        let opcode = (instruction & 0b1111111) as u8;
        match opcode {
            0b0110011 => {
                let rd = ((instruction >> 7) & 0b11111) as u8;
                let funct3 = ((instruction >> 12) & 0b111) as u8;
                let rs1 = ((instruction >> 15) & 0b1111) as u8;
                let rs2 = ((instruction >> 20) & 0b1111) as u8;
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
                let rs1 = ((instruction >> 15) & 0b1111) as u8;
                let rs2 = ((instruction >> 20) & 0b1111) as u8;
                let imm7 = (instruction >> 7) & 0b11111;
                let imm25 = instruction & 0xfff00000;
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
                let imm7 = (instruction >> 7) & 0b11111;
                let imm25 = instruction & 0xfff00000;
                let imm = ((imm25 + (imm7 << 20)) as i32 as u64 >> 20) as i16;
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
                let imm = ((instruction & 0x7ffff800) as i32 as u64 >> 12) as i32;
                J { opcode, rd, imm }
            }
            0b0110111 | 0b0010111 => {
                let rd = ((instruction >> 7) & 0x1F) as u8;
                let imm = ((instruction & 0x7ffff800) as i32 as u64 >> 12) as i32;
                U { opcode, rd, imm }
            }
            _ => {
                eprintln!(
                    "[{:#x}] {:07b} Unknown opcode {}",
                    self.pc,
                    opcode,
                    self.csr[csr::MINSTRET]
                );
                panic!();
            }
        }
    }

    fn execute_instruction(&mut self, instruction: InstructionFormat) {
        eprintln!("[0x{:04x}] {}", self.pc, instruction);

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
                self.set_register(rd, val)
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
                self.set_register(rd, val)
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
                let val = self.memory.read_byte(addr);
                self.set_register(rd, val as u32)
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
                self.memory.write_byte(addr, val)
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
                self.memory.write_word(addr, val)
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
            }
            // jal Jump And Link
            J {
                opcode: 0b1101111,
                rd,
                imm,
            } => {
                self.set_register(rd, self.pc + 4);
                self.pc = self.pc.wrapping_add(imm as u32)
            }
            // auipc Add Upper Imm to PC
            U {
                opcode: 0b0010111,
                rd,
                imm,
            } => {
                // one instruction length less
                let val = self.pc - 4 + ((imm as u32) << 12);
                self.set_register(rd, val)
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
            }
            // ebreak Environment Break
            I {
                opcode: 0b1110011,
                funct3: 0x0,
                imm: 0x1,
                ..
            } => {
                // simply exit the program instead of dropping into the debugger
                process::exit(0);
            }
            _ => {
                eprintln!("Unknown instruction: {:}", instruction);
                todo!()
            }
        }
    }
}

#[derive(Debug)]
enum InstructionFormat {
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
    use crate::ram::Ram;
    use crate::Hart;
    use std::sync::Arc;

    #[test]
    fn addi() {
        let ram = Ram::new(vec![0x13, 0x81, 0x00, 0x7d]);
        let mut m = Hart::new(Arc::new(ram));
        m.tick();
        assert_eq!(m.get_register(2), 2000, "x1 mismatch");
    }

    #[test]
    fn addi_neg() {
        let ram = Ram::new(vec![0x93, 0x01, 0x81, 0xc1]);
        let mut m = Hart::new(Arc::new(ram));
        m.tick();
        assert_eq!(m.get_register(3) as i32, -1000, "x1 mismatch");
    }

    #[test]
    fn it_works() {
        let ram = Ram::new(vec![
            0x93, 0x00, 0x80, 0x3e, // li	ra,1000
            0x13, 0x81, 0x00, 0x7d, // addi	sp,ra,2000
            0x93, 0x01, 0x81, 0xc1, // addi	gp,sp,-1000
            0x13, 0x82, 0x01, 0x83, // addi	tp,gp,-2000
            0x93, 0x02, 0x82, 0x3e, // addi	t0,tp,1000
            0x13, 0x03, 0x00, 0x04, // li	t1,64
            0x13, 0x03, 0x43, 0x00, // addi	t1,t1,4
        ]);
        let mut m = Hart::new(Arc::new(ram));
        m.tick();
        m.tick();
        m.tick();
        m.tick();
        m.tick();
        m.tick();
        m.tick();
        assert_eq!(m.get_register(0), 0, "zero register must be zero");
        assert_eq!(m.get_register(1), 1000, "x1 mismatch");
        assert_eq!(m.get_register(2), 3000, "x2 mismatch");
        assert_eq!(m.get_register(3), 2000, "x3 mismatch");
        assert_eq!(m.get_register(4), 0, "x4 mismatch");
        assert_eq!(m.get_register(5), 1000, "x5 mismatch");
        assert_eq!(m.get_register(6), 0x40 + 4, "deadbeef");
    }
}
