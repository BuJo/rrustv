use std::cmp;
use std::sync::Arc;

use crate::csr;
use crate::csr::Csr;
use crate::device::Device;
use crate::ins::InstructionFormat::{B, I, J, R, S, U};
use crate::ins::{Instruction, InstructionFormat};
use crate::plic::Fault;
use crate::plic::Fault::{Halt, IllegalOpcode};
use crate::reg::reg;
use crate::see;

pub struct Hart<BT: Device> {
    start_pc: usize,

    bus: Arc<BT>,
    registers: [u64; 32],
    pc: usize,
    csr: Csr,

    stop: bool,
}

impl<BT: Device> Hart<BT> {
    pub fn new(id: u64, pc: usize, bus: Arc<BT>) -> Self {
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
            .and_then(|instruction| instruction.decode())
            .and_then(|(ins, decoded)| self.execute_instruction(decoded, ins));

        // simulate passing of time
        self.csr[csr::MCYCLE] += 3;
        self.csr[csr::MINSTRET] += 1;

        res
    }

    pub fn set_register(&mut self, reg: u8, val: u64) {
        match reg {
            0 => {}
            1..=31 => self.registers[reg as usize] = val,
            _ => panic!(),
        }
    }

    pub fn get_register(&self, reg: u8) -> u64 {
        match reg {
            0..=31 => self.registers[reg as usize],
            _ => panic!(),
        }
    }

    fn fetch_instruction(&mut self) -> Result<Instruction, Fault> {
        // Assuming little-endian, the first byte contains the opcode
        let ins = self.bus.read_word(self.pc)?;
        match ins & 0b11 {
            // 32-bit instruction
            0b11 => {
                eprintln!(
                    "[{}] [{:#x}] {:07b} Opcode for ins {:08x} {:032b}",
                    self.csr[csr::MHARTID],
                    self.pc,
                    ins & 0b11,
                    ins,
                    ins
                );
                self.pc += 4;
                Ok(Instruction::IRV32(ins))
            }
            // 16-bit compressed instruction
            _ => {
                let ins = self.bus.read_half(self.pc)?;
                eprintln!(
                    "[{}] [{:#x}] {:02b} Opcode for ins {:04x} {:016b}",
                    self.csr[csr::MHARTID],
                    self.pc,
                    ins & 0b11,
                    ins,
                    ins
                );
                self.pc += 2;
                Ok(Instruction::CRV32(ins))
            }
        }
    }
}

// sign extends the datatype to XLEN
trait SignExtendable {
    fn sext(&self) -> u64;
}

impl SignExtendable for i8 {
    fn sext(&self) -> u64 {
        *self as i64 as u64
    }
}
impl SignExtendable for i16 {
    fn sext(&self) -> u64 {
        *self as i64 as u64
    }
}
impl SignExtendable for i32 {
    fn sext(&self) -> u64 {
        *self as i64 as u64
    }
}
impl SignExtendable for u8 {
    fn sext(&self) -> u64 {
        *self as i8 as i64 as u64
    }
}
impl SignExtendable for u16 {
    fn sext(&self) -> u64 {
        *self as i16 as i64 as u64
    }
}
impl SignExtendable for u32 {
    fn sext(&self) -> u64 {
        *self as i32 as i64 as u64
    }
}
impl SignExtendable for i64 {
    fn sext(&self) -> u64 {
        *self as u64
    }
}

impl<BT: Device> Hart<BT> {
    fn execute_instruction(
        &mut self,
        instruction: InstructionFormat,
        ins: Instruction,
    ) -> Result<(), Fault> {
        match instruction {
            // RV32I

            // add ADD
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
            // addw ADD
            R {
                opcode: 0b0111011,
                rd,
                funct3: 0x0,
                rs1,
                rs2,
                funct7: 0x00,
            } => {
                let val = ((self.get_register(rs1) & 0xFFFFFFFF) as u32)
                    .wrapping_add((self.get_register(rs2) & 0xFFFFFFFF) as u32);
                self.set_register(rd, val.sext());

                self.dbgins(ins, format!("addw\t{},{},{}", reg(rd), reg(rs1), reg(rs2)))
            }
            // sub SUB
            R {
                opcode: 0b0110011,
                rd,
                funct3: 0x0,
                rs1,
                rs2,
                funct7: 0x20,
            } => {
                let val = self.get_register(rs1).wrapping_sub(self.get_register(rs2));
                self.set_register(rd, val);

                self.dbgins(ins, format!("sub\t{},{},{}", reg(rd), reg(rs1), reg(rs2)))
            }
            // subw SUB
            R {
                opcode: 0b0111011,
                rd,
                funct3: 0x0,
                rs1,
                rs2,
                funct7: 0x20,
            } => {
                let val = ((self.get_register(rs1) & 0xFFFFFFFF) as u32)
                    .wrapping_sub((self.get_register(rs2) & 0xFFFFFFFF) as u32);
                self.set_register(rd, val.sext());

                self.dbgins(ins, format!("subw\t{},{},{}", reg(rd), reg(rs1), reg(rs2)))
            }
            // XOR
            R {
                opcode: 0b0110011,
                rd,
                funct3: 0x4,
                rs1,
                rs2,
                funct7: 0x00,
            } => {
                let val = self.get_register(rs1) ^ self.get_register(rs2);
                self.set_register(rd, val);

                self.dbgins(ins, format!("xor\t{},{},{}", reg(rd), reg(rs1), reg(rs2)))
            }
            // OR
            R {
                opcode: 0b0110011,
                rd,
                funct3: 0x6,
                rs1,
                rs2,
                funct7: 0x00,
            } => {
                let val = self.get_register(rs1) | self.get_register(rs2);
                self.set_register(rd, val);

                self.dbgins(ins, format!("or\t{},{},{}", reg(rd), reg(rs1), reg(rs2)))
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
            // sll Shift Left Logical
            R {
                opcode: 0b0110011,
                rd,
                funct3: 0x1,
                rs1,
                rs2,
                funct7: 0x00,
            } => {
                let (val, _) = self
                    .get_register(rs1)
                    .overflowing_shl((self.get_register(rs2) & 0b111111) as u32);
                self.set_register(rd, val);

                self.dbgins(ins, format!("sll\t{},{},{}", reg(rd), reg(rs1), reg(rs2)))
            }
            // sllw Shift Left Logical
            R {
                opcode: 0b0111011,
                rd,
                funct3: 0x1,
                rs1,
                rs2,
                funct7: 0x00,
            } => {
                let (val, _) = ((self.get_register(rs1) & 0xFFFFFFFF) as u32)
                    .overflowing_shl((self.get_register(rs2) & 0b11111) as u32);
                self.set_register(rd, val.sext());

                self.dbgins(ins, format!("sll\t{},{},{}", reg(rd), reg(rs1), reg(rs2)))
            }
            // srl Shift Left Logical
            R {
                opcode: 0b0110011,
                rd,
                funct3: 0x5,
                rs1,
                rs2,
                funct7: 0x00,
            } => {
                let (val, _) = self
                    .get_register(rs1)
                    .overflowing_shr((self.get_register(rs2) & 0b111111) as u32);
                self.set_register(rd, val);

                self.dbgins(ins, format!("srl\t{},{},{}", reg(rd), reg(rs1), reg(rs2)))
            }
            // srlw Shift Left Logical
            R {
                opcode: 0b0111011,
                rd,
                funct3: 0x5,
                rs1,
                rs2,
                funct7: 0x00,
            } => {
                let (val, _) = ((self.get_register(rs1) & 0xFFFFFFFF) as u32)
                    .overflowing_shr((self.get_register(rs2) & 0b11111) as u32);
                self.set_register(rd, val.sext());

                self.dbgins(ins, format!("srl\t{},{},{}", reg(rd), reg(rs1), reg(rs2)))
            }
            // sra Shift Right Arith
            R {
                opcode: 0b0110011,
                rd,
                funct3: 0x5,
                rs1,
                rs2,
                funct7: 0x20,
            } => {
                let (val, _) = (self.get_register(rs1) as i64)
                    .overflowing_shr((self.get_register(rs2) & 0b111111) as u32);
                self.set_register(rd, val as u64);

                self.dbgins(ins, format!("sra\t{},{},{}", reg(rd), reg(rs1), reg(rs2)))
            }
            // sraw Shift Right Arith
            R {
                opcode: 0b0111011,
                rd,
                funct3: 0x5,
                rs1,
                rs2,
                funct7: 0x20,
            } => {
                let (val, _) = ((self.get_register(rs1) & 0xFFFFFFFF) as i32)
                    .overflowing_shr((self.get_register(rs2) & 0b11111) as u32);
                self.set_register(rd, val as u64);

                self.dbgins(ins, format!("sra\t{},{},{}", reg(rd), reg(rs1), reg(rs2)))
            }
            // slt Set Less Than
            R {
                opcode: 0b0110011,
                rd,
                funct3: 0x2,
                rs1,
                rs2,
                funct7: 0x00,
            } => {
                let val = if (self.get_register(rs1) as i64) < (self.get_register(rs2) as i64) {
                    1
                } else {
                    0
                };
                self.set_register(rd, val.sext());

                self.dbgins(ins, format!("slt\t{},{},{}", reg(rd), reg(rs1), reg(rs2)))
            }
            // sltu Set Less Than (U, zero extends)
            R {
                opcode: 0b0110011,
                rd,
                funct3: 0x3,
                rs1,
                rs2,
                funct7: 0x00,
            } => {
                let val = if self.get_register(rs1) < self.get_register(rs2) {
                    1
                } else {
                    0
                };
                self.set_register(rd, val as u64);

                self.dbgins(ins, format!("sltu\t{},{},{}", reg(rd), reg(rs1), reg(rs2)))
            }

            // addi ADD immediate
            I {
                opcode: 0b0010011,
                rd,
                funct3: 0x0,
                rs1,
                imm,
            } => {
                let val = self.get_register(rs1).wrapping_add(imm.sext());

                if rd == 0 {
                    self.dbgins(ins, "nop".to_string())
                } else {
                    self.set_register(rd, val);

                    self.dbgins(
                        ins,
                        format!("add\t{},{},{} # {:x}", reg(rd), reg(rs1), imm, val),
                    )
                }
            }
            // addiw ADD immediate word
            I {
                opcode: 0b0011011,
                rd,
                funct3: 0x0,
                rs1,
                imm,
            } => {
                if imm == 0 {
                    let extended = (self.get_register(rs1) & 0xFFFFFFFF) as i32;
                    self.set_register(rd, extended.sext());

                    self.dbgins(ins, format!("sext.w\t{},{}", reg(rd), reg(rs1)))
                } else {
                    let val = ((self.get_register(rs1) & 0xFFFFFFFF) as u32)
                        .wrapping_add(imm as i32 as u32);
                    self.set_register(rd, val.sext());

                    self.dbgins(
                        ins,
                        format!("addw\t{},{},{} # {:x}", reg(rd), reg(rs1), imm, val),
                    )
                }
            }
            // xori XOR immediate
            I {
                opcode: 0b0010011,
                rd,
                funct3: 0x4,
                rs1,
                imm,
            } => {
                let val = self.get_register(rs1) ^ imm.sext();
                self.set_register(rd, val);

                self.dbgins(
                    ins,
                    format!("xor\t{},{},{} # {:x}", reg(rd), reg(rs1), imm, val),
                )
            }
            // ori OR immediate
            I {
                opcode: 0b0010011,
                rd,
                funct3: 0x6,
                rs1,
                imm,
            } => {
                let val = self.get_register(rs1) | imm as u64;
                self.set_register(rd, val);

                self.dbgins(
                    ins,
                    format!("or\t{},{},{} # {:x}", reg(rd), reg(rs1), imm, val),
                )
            }
            // andi AND immediate
            I {
                opcode: 0b0010011,
                rd,
                funct3: 0x7,
                rs1,
                imm,
            } => {
                let val = self.get_register(rs1) & imm as u64;
                self.set_register(rd, val);

                self.dbgins(
                    ins,
                    format!("and\t{},{},{} # {:x}", reg(rd), reg(rs1), imm, val),
                )
            }
            // slli Shift Left Logical Imm
            I {
                opcode: 0b0010011,
                rd,
                funct3: 0x1,
                rs1,
                imm,
            } => {
                let rs1val = self.get_register(rs1);
                let shift = (imm & 0b111111) as u32;
                let (val, _) = rs1val.overflowing_shl(shift);
                self.set_register(rd, val);

                self.dbgins(ins, format!("sll\t{},{},{:#x}", reg(rd), reg(rs1), imm))
            }
            // slliw Shift Left Logical Imm
            I {
                opcode: 0b0011011,
                rd,
                funct3: 0x1,
                rs1,
                imm,
            } => {
                let (val, _) = ((self.get_register(rs1) & 0xFFFFFFFF) as u32)
                    .overflowing_shl((imm & 0b11111) as u32);
                self.set_register(rd, val.sext());

                self.dbgins(ins, format!("sll\t{},{},{:#x}", reg(rd), reg(rs1), imm))
            }
            // srli Shift Right Logical Imm
            I {
                opcode: 0b0010011,
                rd,
                funct3: 0x5,
                rs1,
                imm,
            } if ((imm as u16) >> 6) == 0x00 => {
                let (val, _) = self
                    .get_register(rs1)
                    .overflowing_shr((imm & 0b111111) as u32);
                self.set_register(rd, val);

                self.dbgins(
                    ins,
                    format!("srl\t{},{},{:#x} # {:x}", reg(rd), reg(rs1), imm, val),
                )
            }
            // srliw Shift Right Logical Imm
            I {
                opcode: 0b0011011,
                rd,
                funct3: 0x5,
                rs1,
                imm,
            } if ((imm as u16) >> 6) == 0x00 => {
                let (val, _) = ((self.get_register(rs1) & 0xFFFFFFFF) as u32)
                    .overflowing_shr((imm & 0b11111) as u32);
                self.set_register(rd, val.sext());

                self.dbgins(
                    ins,
                    format!("srlw\t{},{},{:#x} # {:x}", reg(rd), reg(rs1), imm, val),
                )
            }
            // srai Shift Right Arith Imm
            I {
                opcode: 0b0010011,
                rd,
                funct3: 0x5,
                rs1,
                imm,
            } if ((imm as u16) >> 6) == 0x10 => {
                let shamt = (imm & 0b111111) as u32;
                let (val, _) = (self.get_register(rs1) as i64).overflowing_shr(shamt);
                self.set_register(rd, val.sext());

                self.dbgins(
                    ins,
                    format!("sra\t{},{},{:#x} # {:x}", reg(rd), reg(rs1), shamt, val),
                )
            }
            // sraiw Shift Right Arith Imm
            I {
                opcode: 0b0011011,
                rd,
                funct3: 0x5,
                rs1,
                imm,
            } if ((imm as u16) >> 6) == 0x10 => {
                let (val, _) = ((self.get_register(rs1) & 0xFFFFFFFF) as i32)
                    .overflowing_shr((imm & 0b11111) as u32);
                self.set_register(rd, val.sext());

                self.dbgins(
                    ins,
                    format!(
                        "sraw\t{},{},{:#x} # {:x}",
                        reg(rd),
                        reg(rs1),
                        (imm & 0b11111),
                        val
                    ),
                )
            }
            // slti Set Less Than Imm
            I {
                opcode: 0b0010011,
                rd,
                funct3: 0x2,
                rs1,
                imm,
            } => {
                let val = if (self.get_register(rs1) as i64) < (imm as i64) {
                    1
                } else {
                    0
                };
                self.set_register(rd, val);

                self.dbgins(
                    ins,
                    format!("slti\t{},{},{} # {:x}", reg(rd), reg(rs1), imm, val),
                )
            }
            // sltiu Set Less Than Imm (U, zero extends)
            I {
                opcode: 0b0010011,
                rd,
                funct3: 0x3,
                rs1,
                imm,
            } => {
                let val = if self.get_register(rs1) < (imm as u64) {
                    1
                } else {
                    0
                };
                self.set_register(rd, val);

                self.dbgins(
                    ins,
                    format!("sltiu\t{},{},{} # {:x}", reg(rd), reg(rs1), imm, val),
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
                let addr = (self.get_register(rs1).wrapping_add(imm.sext())) as usize;
                let val = self.bus.read_byte(addr)? as i8;
                self.set_register(rd, val.sext());

                self.dbgins(ins, format!("lb\t{},{}({})", reg(rd), imm, reg(rs1)))
            }
            // lh Load Half
            I {
                opcode: 0b0000011,
                rd,
                funct3: 0x1,
                rs1,
                imm,
            } => {
                let addr = (self.get_register(rs1).wrapping_add(imm.sext())) as usize;
                let val = self.bus.read_half(addr)?;
                self.set_register(rd, val.sext());

                self.dbgins(ins, format!("lh\t{},{}({})", reg(rd), imm, reg(rs1)))
            }
            // lw Load Word
            I {
                opcode: 0b0000011,
                rd,
                funct3: 0x2,
                rs1,
                imm,
            } => {
                let addr = (self.get_register(rs1).wrapping_add(imm.sext())) as usize;

                self.dbgins(ins, format!("lw\t{},{}({})", reg(rd), imm, reg(rs1)));

                let val = self.bus.read_word(addr)?;
                self.set_register(rd, val.sext());
            }
            // ld Load Double
            I {
                opcode: 0b0000011,
                rd,
                funct3: 0x3,
                rs1,
                imm,
            } => {
                let addr = (self.get_register(rs1).wrapping_add(imm.sext())) as usize;

                self.dbgins(ins, format!("ld\t{},{}({})", reg(rd), imm, reg(rs1)));

                let val = self.bus.read_double(addr)?;
                self.set_register(rd, val);
            }
            // lbu Load Byte (U, zero extends)
            I {
                opcode: 0b0000011,
                rd,
                funct3: 0x4,
                rs1,
                imm,
            } => {
                let addr = (self.get_register(rs1).wrapping_add(imm.sext())) as usize;
                let val = self.bus.read_byte(addr)?;
                self.set_register(rd, val as u64);

                self.dbgins(ins, format!("lbu\t{},{},{:#x}", reg(rd), reg(rs1), imm))
            }
            // lhu Load Half (U, zero extends)
            I {
                opcode: 0b0000011,
                rd,
                funct3: 0x5,
                rs1,
                imm,
            } => {
                let addr = (self.get_register(rs1).wrapping_add(imm as u64)) as usize;
                let val = self.bus.read_half(addr)?;
                self.set_register(rd, val as u64);

                self.dbgins(ins, format!("lhu\t{},{},{:#x}", reg(rd), reg(rs1), imm))
            }
            // lwu Load Word (U, zero extends)
            I {
                opcode: 0b0000011,
                rd,
                funct3: 0x6,
                rs1,
                imm,
            } => {
                let addr = (self.get_register(rs1).wrapping_add(imm as u64)) as usize;
                let val = self.bus.read_word(addr)?;
                self.set_register(rd, val as u64);

                self.dbgins(ins, format!("lwu\t{},{},{:#x}", reg(rd), reg(rs1), imm))
            }

            // sb Store Byte
            S {
                opcode: 0b0100011,
                funct3: 0x0,
                rs1,
                rs2,
                imm,
            } => {
                let addr = (self.get_register(rs1).wrapping_add(imm.sext())) as usize;
                let val = (self.get_register(rs2) & 0xFF) as u8;

                self.dbgins(ins, format!("sb\t{},{}({})", reg(rs2), imm, reg(rs1)));
                return self.bus.write_byte(addr, val);
            }
            // sh Store Half
            S {
                opcode: 0b0100011,
                funct3: 0x1,
                rs1,
                rs2,
                imm,
            } => {
                let addr = (self.get_register(rs1).wrapping_add(imm.sext())) as usize;
                let val = (self.get_register(rs2) & 0xFFFF) as u16;

                self.dbgins(ins, format!("sh\t{},{}({})", reg(rs2), imm, reg(rs1)));
                return self.bus.write_half(addr, val);
            }
            // sw Store Word
            S {
                opcode: 0b0100011,
                funct3: 0x2,
                rs1,
                rs2,
                imm,
            } => {
                let addr = (self.get_register(rs1).wrapping_add(imm.sext())) as usize;
                let val = (self.get_register(rs2) & 0xFFFFFFFF) as u32;

                self.dbgins(ins, format!("sw\t{},{}({})", reg(rs2), imm, reg(rs1)));
                return self.bus.write_word(addr, val);
            }
            // sd Store Double
            S {
                opcode: 0b0100011,
                funct3: 0x3,
                rs1,
                rs2,
                imm,
            } => {
                let addr = (self.get_register(rs1).wrapping_add(imm.sext())) as usize;
                let val = self.get_register(rs2);

                self.dbgins(ins, format!("sd\t{},{}({})", reg(rs2), imm, reg(rs1)));
                return self.bus.write_double(addr, val);
            }
            // beq Branch ==
            B {
                opcode: 0b1100011,
                funct3: 0x0,
                rs1,
                rs2,
                imm,
            } => {
                let isize = ins.size();
                let target = self.pc.wrapping_add(imm as usize).wrapping_sub(isize);
                self.dbgins(ins, format!("beq\t{},{},{:x}", reg(rs1), reg(rs2), target));

                if self.get_register(rs1) == self.get_register(rs2) {
                    self.pc = target;
                }
            }
            // bne Branch !=
            B {
                opcode: 0b1100011,
                funct3: 0x1,
                rs1,
                rs2,
                imm,
            } => {
                let isize = ins.size();
                let target = self.pc.wrapping_add(imm as usize).wrapping_sub(isize);
                self.dbgins(ins, format!("bne\t{},{},{:x}", reg(rs1), reg(rs2), target));

                if self.get_register(rs1) != self.get_register(rs2) {
                    self.pc = target;
                }
            }
            // blt Branch <
            B {
                opcode: 0b1100011,
                funct3: 0x4,
                rs1,
                rs2,
                imm,
            } => {
                let isize = ins.size();
                let target = self.pc.wrapping_add(imm as usize).wrapping_sub(isize);
                self.dbgins(ins, format!("blt\t{},{},{:x}", reg(rs1), reg(rs2), target));

                if (self.get_register(rs1) as i64) < (self.get_register(rs2) as i64) {
                    self.pc = target;
                }
            }
            // bge Branch >=
            B {
                opcode: 0b1100011,
                funct3: 0x5,
                rs1,
                rs2,
                imm,
            } => {
                let isize = ins.size();
                let target = self.pc.wrapping_add(imm as usize).wrapping_sub(isize);
                self.dbgins(ins, format!("bge\t{},{},{:x}", reg(rs1), reg(rs2), target));

                if (self.get_register(rs1) as i64) >= (self.get_register(rs2) as i64) {
                    self.pc = target;
                }
            }
            // bgltu Branch < (U, zero extends)
            B {
                opcode: 0b1100011,
                funct3: 0x6,
                rs1,
                rs2,
                imm,
            } => {
                let isize = ins.size();
                let target = self.pc.wrapping_add(imm as usize).wrapping_sub(isize);
                self.dbgins(
                    ins,
                    format!("bgltu\t{},{},{:x}", reg(rs1), reg(rs2), target),
                );

                if self.get_register(rs1) < self.get_register(rs2) {
                    self.pc = target;
                }
            }
            // bgeu Branch >= (U, zero extends)
            B {
                opcode: 0b1100011,
                funct3: 0x7,
                rs1,
                rs2,
                imm,
            } => {
                let isize = ins.size();
                let target = self.pc.wrapping_add(imm as usize).wrapping_sub(isize);
                self.dbgins(ins, format!("bgeu\t{},{},{:x}", reg(rs1), reg(rs2), target));

                if self.get_register(rs1) >= self.get_register(rs2) {
                    self.pc = target;
                }
            }

            // jal Jump And Link
            J {
                opcode: 0b1101111,
                rd,
                imm,
            } => {
                let isize = ins.size();
                let target = self.pc.wrapping_add(imm as usize).wrapping_sub(isize);
                self.dbgins(ins, format!("jal\t{},{:x}", reg(rd), target));

                self.set_register(rd, self.pc as u64);
                self.pc = target;
            }
            // jalr Jump And Link Reg
            I {
                opcode: 0b1100111,
                rd,
                funct3: 0x0,
                rs1,
                imm,
            } => {
                let target = self.get_register(rs1).wrapping_add(imm as u64);
                // Clear last bit: Spec (V 2.1, p. 5), align to 16 bit parcels
                let target = target & 0xFFFF_FFFE;

                self.dbgins(ins, format!("jalr\t{},{}({})", reg(rd), imm, reg(rs1)));

                self.set_register(rd, self.pc as u64);
                self.pc = target as usize;
            }

            // lui Load Upper Imm
            U {
                opcode: 0b0110111,
                rd,
                imm,
            } => {
                let val = (imm << 12) as i64 as u64;
                self.set_register(rd, val);

                self.dbgins(ins, format!("lui\t{},{:#x}", reg(rd), imm))
            }
            // auipc Add Upper Imm to PC
            U {
                opcode: 0b0010111,
                rd,
                imm,
            } => {
                let val = (imm << 12) as i64 as u64;
                let val = (self.pc as u64 - 4).wrapping_add(val);
                self.set_register(rd, val);

                self.dbgins(ins, format!("auipc\t{},{:#x}", reg(rd), imm))
            }

            // RV32 Zifencei
            // Fence
            I {
                opcode: 0b0001111,
                funct3: 0x0,
                rd: 0x0,
                rs1: 0x0,
                imm,
            } => {
                let pred = (imm >> 4) & 0b1111;
                let succ = imm & 0b1111;
                self.dbgins(ins, format!("fence\t{},{}", pred, succ))
            }
            // Fence.I
            I {
                opcode: 0b0001111,
                funct3: 0x1,
                rd: 0x0,
                rs1: 0x0,
                imm: 0,
            } => {
                // For now, all accesses to addresses go through locking, ignore fence
                self.dbgins(ins, "fence unknown,unknown".to_string())
            }

            // ecall Environment Call
            I {
                opcode: 0b1110011,
                funct3: 0x0,
                imm: 0x0,
                ..
            } => {
                // We're unprivileged machine mode, no need to check SEDELEG

                self.dbgins(ins, "ecall".to_string());

                // For now, ignore SEE errors
                let _ = see::call(self);
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

            // RV32/RV64 Zicsr
            // csrrw Atomic Read/Write CSR
            I {
                opcode: 0b1110011,
                rd,
                funct3: 0x1,
                rs1,
                imm,
            } => {
                let csr = (imm as u16 & 0xFFF) as usize;

                if rd != 0 {
                    self.set_register(rd, self.csr[csr]);
                }
                self.csr[csr] = self.get_register(rs1);

                self.dbgins(
                    ins,
                    format!("csrrw\t{},{},{}", reg(rd), Csr::name(csr), reg(rs1)),
                )
            }
            // csrrs Atomic Read and Set Bits in CSR
            I {
                opcode: 0b1110011,
                rd,
                funct3: 0x2,
                rs1,
                imm,
            } => {
                let csr = (imm as u16 & 0xFFF) as usize;

                self.set_register(rd, self.csr[csr]);

                if rs1 != 0 {
                    self.csr[csr] |= self.get_register(rs1);
                }

                self.dbgins(
                    ins,
                    format!("csrrs\t{},{},{}", reg(rd), Csr::name(csr), reg(rs1)),
                )
            }
            // csrrc Atomic Read and Clear Bits in CSR
            I {
                opcode: 0b1110011,
                rd,
                funct3: 0x3,
                rs1,
                imm,
            } => {
                let csr = (imm as u16 & 0xFFF) as usize;
                if rd != 0 {
                    self.set_register(rd, self.csr[csr]);
                }

                if rs1 != 0 {
                    self.csr[csr] &= !self.get_register(rs1);
                }

                self.dbgins(
                    ins,
                    format!("csrrc\t{},{},{}", reg(rd), Csr::name(csr), reg(rs1)),
                )
            }
            // csrrwi
            I {
                opcode: 0b1110011,
                rd,
                funct3: 0x5,
                rs1,
                imm,
            } => {
                let csr = (imm as u16 & 0xFFF) as usize;
                let imm = rs1 as u64;

                self.dbgins(
                    ins,
                    format!("csrrwi\t{},{},{}", reg(rd), Csr::name(csr), imm),
                );

                if rd != 0 {
                    self.set_register(rd, self.csr[csr]);
                }
                self.csr[csr] = imm;
            }
            // csrrsi
            I {
                opcode: 0b1110011,
                rd,
                funct3: 0x6,
                rs1,
                imm,
            } => {
                let csr = (imm as u16 & 0xFFF) as usize;
                let imm = rs1 as u64;

                self.dbgins(
                    ins,
                    format!("csrrsi\t{},{},{}", reg(rd), Csr::name(csr), imm),
                );

                self.set_register(rd, self.csr[csr]);

                if rs1 != 0 {
                    self.csr[csr] |= imm;
                }
            }
            // csrrci
            I {
                opcode: 0b1110011,
                rd,
                funct3: 0x7,
                rs1,
                imm,
            } => {
                let csr = (imm as u16 & 0xFFF) as usize;
                let imm = rs1 as u64;

                self.dbgins(
                    ins,
                    format!("csrrci\t{},{},{}", reg(rd), Csr::name(csr), imm),
                );

                if rd != 0 {
                    self.set_register(rd, self.csr[csr]);
                }

                if rs1 != 0 {
                    self.csr[csr] &= !imm;
                }
            }

            // Supervisor Memory-Management Instructions
            // sfence.vma Atomic Read and Clear Bits in CSR
            R {
                opcode: 0b1110011,
                rd,
                funct3: 0x0,
                rs1,
                rs2,
                ..
            } => self.dbgins(
                ins,
                format!(
                    "system\t{},{},{} # {:08x}",
                    reg(rd),
                    reg(rs1),
                    reg(rs2),
                    ins
                ),
            ),

            // Atomics
            R {
                opcode: 0b0101111,
                rd,
                funct3: 0x2,
                rs1,
                rs2,
                funct7,
            } => {
                let funct5 = funct7 >> 2;
                let _aq = (funct7 >> 1) & 0b1;
                let _rl = funct7 & 0b1;

                let addr = self.get_register(rs1) as usize;
                let val = self.bus.read_word(addr)?;
                let rs2val = (self.get_register(rs2) & 0xFFFFFFFF) as u32;
                let new = match funct5 {
                    // amoswap.w
                    0x01 => {
                        self.dbgins(
                            ins,
                            format!("amoswap.w\t{},{},({})", reg(rd), reg(rs2), reg(rs1)),
                        );
                        let rdval = self.get_register(rd);
                        self.set_register(rs2, rdval);
                        rs2val
                    }
                    // amoadd.w
                    0x00 => {
                        self.dbgins(
                            ins,
                            format!("amoadd.w\t{},{},({})", reg(rd), reg(rs2), reg(rs1)),
                        );

                        val.wrapping_add(rs2val)
                    }
                    // amoand.w
                    0x0C => {
                        self.dbgins(
                            ins,
                            format!("amoand.w\t{},{},({})", reg(rd), reg(rs2), reg(rs1)),
                        );
                        val & rs2val
                    }
                    // amoor.w
                    0x08 => {
                        self.dbgins(
                            ins,
                            format!("amoor.w\t{},{},({})", reg(rd), reg(rs2), reg(rs1)),
                        );
                        val | rs2val
                    }
                    // amoxor.w
                    0x04 => {
                        self.dbgins(
                            ins,
                            format!("amoxor.w\t{},{},({})", reg(rd), reg(rs2), reg(rs1)),
                        );
                        val ^ rs2val
                    }
                    // amomax.w
                    0x14 => {
                        self.dbgins(
                            ins,
                            format!("amomax.w\t{},{},({})", reg(rd), reg(rs2), reg(rs1)),
                        );
                        cmp::max(val as i32, rs2val as i32) as u32
                    }
                    // amomin.w
                    0x10 => {
                        self.dbgins(
                            ins,
                            format!("amomin.w\t{},{},({})", reg(rd), reg(rs2), reg(rs1)),
                        );
                        cmp::min(val as i32, rs2val as i32) as u32
                    }
                    // amomaxu.w
                    0x1C => {
                        self.dbgins(
                            ins,
                            format!("amomaxu.w\t{},{},({})", reg(rd), reg(rs2), reg(rs1)),
                        );
                        cmp::max(val, rs2val)
                    }
                    // amominu.w
                    0x18 => {
                        self.dbgins(
                            ins,
                            format!("amominu.w\t{},{},({})", reg(rd), reg(rs2), reg(rs1)),
                        );
                        cmp::min(val, rs2val)
                    }
                    _ => return Err(IllegalOpcode(ins)),
                };

                self.set_register(rd, val.sext());
                self.bus.write_word(addr, new)?;
            }
            R {
                opcode: 0b0101111,
                rd,
                funct3: 0x3,
                rs1,
                rs2,
                funct7,
            } => {
                let funct5 = funct7 >> 2;
                let _aq = (funct7 >> 1) & 0b1;
                let _rl = funct7 & 0b1;

                let addr = self.get_register(rs1) as usize;
                let val = self.bus.read_double(addr)?;
                let rs2val = self.get_register(rs2);
                let new = match funct5 {
                    // amoswap.d
                    0x01 => {
                        self.dbgins(
                            ins,
                            format!("amoswap.d\t{},{},({})", reg(rd), reg(rs2), reg(rs1)),
                        );
                        let rdval = self.get_register(rd);
                        self.set_register(rs2, rdval);
                        rs2val
                    }
                    // amoadd.d
                    0x00 => {
                        self.dbgins(
                            ins,
                            format!("amoadd.d\t{},{},({})", reg(rd), reg(rs2), reg(rs1)),
                        );

                        val.wrapping_add(rs2val)
                    }
                    // amoand.d
                    0x0C => {
                        self.dbgins(
                            ins,
                            format!("amoand.d\t{},{},({})", reg(rd), reg(rs2), reg(rs1)),
                        );
                        val & rs2val
                    }
                    // amoor.d
                    0x08 => {
                        self.dbgins(
                            ins,
                            format!("amoor.d\t{},{},({})", reg(rd), reg(rs2), reg(rs1)),
                        );
                        val | rs2val
                    }
                    // amoxor.d
                    0x04 => {
                        self.dbgins(
                            ins,
                            format!("amoxor.d\t{},{},({})", reg(rd), reg(rs2), reg(rs1)),
                        );
                        val ^ rs2val
                    }
                    // amomax.d
                    0x14 => {
                        self.dbgins(
                            ins,
                            format!("amomax.d\t{},{},({})", reg(rd), reg(rs2), reg(rs1)),
                        );
                        cmp::max(val as i64, rs2val as i64) as u64
                    }
                    // amomin.d
                    0x10 => {
                        self.dbgins(
                            ins,
                            format!("amomin.d\t{},{},({})", reg(rd), reg(rs2), reg(rs1)),
                        );
                        cmp::min(val as i64, rs2val as i64) as u64
                    }
                    // amomaxu.d
                    0x1C => {
                        self.dbgins(
                            ins,
                            format!("amomaxu.d\t{},{},({})", reg(rd), reg(rs2), reg(rs1)),
                        );
                        cmp::max(val, rs2val)
                    }
                    // amominu.d
                    0x18 => {
                        self.dbgins(
                            ins,
                            format!("amominu.d\t{},{},({})", reg(rd), reg(rs2), reg(rs1)),
                        );
                        cmp::min(val, rs2val)
                    }
                    _ => return Err(IllegalOpcode(ins)),
                };

                self.set_register(rd, val);
                self.bus.write_double(addr, new)?;
            }

            _ => {
                eprintln!(
                    "[{}] Unknown instruction: {:}",
                    self.csr[csr::MHARTID],
                    instruction
                );
                return Err(Fault::MemoryFault(self.pc));
            }
        };
        Ok(())
    }

    fn dbgins(&self, ins: Instruction, asm: String) {
        match ins {
            Instruction::IRV32(ins) => {
                eprintln!("{:08x}:\t{:08x}          \t{}", self.pc - 4, ins, asm)
            }
            Instruction::CRV32(ins) => {
                eprintln!("{:08x}:\t{:04x}                \t{}", self.pc - 2, ins, asm)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::bus::Bus;
    use crate::hart::Hart;
    use crate::ins::{Instruction, InstructionFormat};
    use crate::ram::Ram;
    use crate::reg::treg;
    use crate::rom::Rom;

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
        assert_eq!(m.get_register(3) as i64, -1000, "x1 mismatch");
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
    fn test_auipc_800032c0() {
        let ins = Instruction::IRV32(0x00001f17);
        let mut m = hart();
        m.pc = 0x800032c0 + 4;

        let decoded = ins.decode().expect("decode").1;
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
    fn test_beq_80000134() {
        // j	80000938
        let ins = Instruction::IRV32(0x0050006f);
        let mut m = hart();
        m.pc = 0x80000134;

        let decoded = ins.decode().expect("decode").1;
        println!("{:032b} {}", 0x0050006f, decoded);
        match decoded {
            InstructionFormat::J { opcode, rd, imm } => {
                assert_eq!(opcode, 0b1101111, "opcode wrong");
                assert_eq!(rd, treg("zero"), "rd wrong");
                assert_eq!(imm, 2052, "imm wrong");
            }
            _ => assert!(false, "not sw"),
        }

        m.execute_instruction(decoded, ins).expect("execute");
    }

    #[test]
    fn test_beq_80000938() {
        // beq	s3,s3,80000138
        let ins = Instruction::IRV32(0x813980e3);
        let mut m = hart();
        m.pc = 0x80000134;

        let decoded = ins.decode().expect("decode").1;
        println!("{:032b} {}", 0x813980e3u64, decoded);
        match decoded {
            InstructionFormat::B {
                opcode,
                funct3,
                rs1,
                rs2,
                imm,
            } => {
                assert_eq!(opcode, 0b1100011, "opcode wrong");
                assert_eq!(funct3, 0x0, "funct3 wrong");
                assert_eq!(rs1, treg("s3"), "rs1 wrong");
                assert_eq!(rs2, treg("s3"), "rs1 wrong");
                assert_eq!(imm, -2048, "imm wrong");
            }
            _ => assert!(false, "not sw"),
        }

        m.set_register(treg("s3"), 0x55555555);
        m.execute_instruction(decoded, ins).expect("execute");
    }

    #[test]
    fn test_rv64_sll_80000404() {
        // beq	s3,s3,80000138
        let ins = Instruction::IRV32(0x026b1b13);
        let mut m = hart();
        m.pc = 0x80000404;
        m.set_register(treg("s6"), 0x40);
        assert_eq!(m.get_register(treg("s6")), 0x40);

        let decoded = ins.decode().expect("decode").1;
        match decoded {
            InstructionFormat::I {
                opcode,
                rd,
                funct3,
                rs1,
                imm,
            } => {
                assert_eq!(opcode, 0b0010011, "opcode wrong");
                assert_eq!(funct3, 0x1, "funct3 wrong");
                assert_eq!(rd, treg("s6"), "rd wrong");
                assert_eq!(rs1, treg("s6"), "rs1 wrong");
                assert_eq!(imm, 38, "imm wrong");
            }
            _ => assert!(false),
        }

        m.execute_instruction(decoded, ins).expect("execute");

        assert_eq!(m.get_register(treg("s6")), 0x40 << 38);
    }

    #[test]
    fn test_addw() {
        // beq	s3,s3,80000138
        let ins = Instruction::IRV32(0x015a81bb);
        let mut m = hart();
        m.pc = 0x80000404;
        m.set_register(treg("gp"), 0x0);
        m.set_register(treg("s5"), 0x1);

        let decoded = ins.decode().expect("decode").1;
        match decoded {
            InstructionFormat::R {
                opcode,
                rd,
                funct3,
                rs1,
                rs2,
                funct7,
            } => {
                assert_eq!(opcode, 0b0111011, "opcode wrong");
                assert_eq!(funct3, 0x0, "funct3 wrong");
                assert_eq!(rd, treg("gp"), "rd wrong");
                assert_eq!(rs1, treg("s5"), "rs1 wrong");
                assert_eq!(rs2, treg("s5"), "rs1 wrong");
                assert_eq!(funct7, 0x0, "funct7 wrong");
            }
            _ => assert!(false),
        }

        m.execute_instruction(decoded, ins).expect("execute");

        assert_eq!(m.get_register(treg("gp")), 0x2);
    }

    #[test]
    fn test_li() {
        // beq	s3,s3,80000138
        let ins = Instruction::IRV32(0x00000413);
        let mut m = hart();
        m.pc = 0x80000418;
        m.set_register(treg("s0"), 0xdeadbeef);

        let decoded = ins.decode().expect("decode").1;
        match decoded {
            InstructionFormat::I {
                opcode,
                rd,
                funct3,
                rs1,
                imm,
            } => {
                assert_eq!(opcode, 0b0010011, "opcode wrong");
                assert_eq!(funct3, 0x0, "funct3 wrong");
                assert_eq!(rd, treg("s0"), "rd wrong");
                assert_eq!(rs1, treg("zero"), "rs1 wrong");
                assert_eq!(imm, 0, "imm wrong");
            }
            _ => assert!(false),
        }

        m.execute_instruction(decoded, ins).expect("execute");

        assert_eq!(m.get_register(treg("gp")), 0x0);
    }
}
