use std::fmt;

use crate::plic::Fault::{self, IllegalOpcode, InstructionDecodingError};

use self::InstructionFormat::{B, I, J, R, S, U};

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

#[derive(Clone, Copy, Debug)]
pub enum Instruction {
    IRV32(u32),
    CRV32(u16),
}

impl Instruction {
    pub fn size(&self) -> usize {
     match self {
         Instruction::IRV32(_) => 4,
         Instruction::CRV32(_) => 2
     }
    }

    pub fn decode(self) -> Result<(Instruction, InstructionFormat), Fault> {
        let res = match self {
            Instruction::IRV32(instruction) => Instruction::decode_32(instruction),
            Instruction::CRV32(instruction) => Instruction::decode_16(instruction),
        };
        res.map(|d| (self, d)).map_err(|_| IllegalOpcode(self))
    }

    fn decode_32(instruction: u32) -> Result<InstructionFormat, Fault> {
        let opcode = (instruction & 0b1111111) as u8;
        let decoded = match opcode {
            0b0110011 | 0b0101111 => {
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
            0b0010011 | 0b0000011 | 0b1100111 | 0b1110011 | 0b0001111 => {
                let rd = ((instruction & 0x0F80) >> 7) as u8;
                let funct3 = ((instruction & 0x7000) >> 12) as u8;
                let rs1 = ((instruction & 0xF8000) >> 15) as u8;
                let imm = ((instruction as i32) >> 20) as i16;
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
                let rs1 = ((instruction >> 15) & 0b11111) as u8;
                let rs2 = ((instruction >> 20) & 0b11111) as u8;
                let imm = ((instruction & 0x8000_0000) >> 19)
                    | ((instruction & 0x7e00_0000) >> 20)
                    | ((instruction & 0x0000_0f00) >> 7)
                    | ((instruction & 0x0000_0080) << 4);
                let imm = (((imm << 19) as i32) >> 19) as i16;

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
                return Err(InstructionDecodingError);
            }
        };

        Ok(decoded)
    }

    fn decode_16(instruction: u16) -> Result<InstructionFormat, Fault> {
        const RVC_REG_OFFSET: u8 = 0x8;

        let op = instruction & 0b11;

        let ins = match op {
            // C0
            0b00 => {
                let funct3 = instruction >> 13;
                match funct3 {
                    // CL-Type: c.lw -> lw rd', (4*imm)(sp)
                    0b010 => {
                        let rd = ((instruction >> 2) & 0b111) as u8;
                        let rs1 = ((instruction >> 7) & 0b111) as u8;
                        let imm = (((instruction >> 6) as u8 & 0b1) << 3)
                            | (((instruction >> 5) as u8 & 0b1) << 7)
                            | (((instruction >> 10) as u8 & 0b111) << 4);
                        let imm = imm >> 1;
                        I {
                            opcode: 0b0000011,
                            rd: rd + RVC_REG_OFFSET,
                            funct3: 0x2,
                            rs1: rs1 + RVC_REG_OFFSET,
                            imm: imm as i16,
                        }
                    }
                    // CS-Type: c.sw -> sw rs1', (4*imm)(rs2')
                    0b110 => {
                        let rs1 = ((instruction >> 7) & 0b111) as u8;
                        let rs2 = ((instruction >> 2) & 0b111) as u8;
                        let imm = (((instruction >> 6) as u8 & 0b1) << 3)
                            | (((instruction >> 5) as u8 & 0b1) << 7)
                            | (((instruction >> 10) as u8 & 0b111) << 4);
                        let imm = imm >> 1;
                        S {
                            opcode: 0b0100011,
                            funct3: 0x2,
                            rs1: rs1 + RVC_REG_OFFSET,
                            rs2: rs2 + RVC_REG_OFFSET,
                            imm: imm as i16,
                        }
                    }
                    // CIW-Type: c.addi4spn -> addi rd', x2, imm
                    0b000 => {
                        let rd = ((instruction >> 2) & 0b111) as u8;
                        //  nzuimm[5:4|9:6|2|3]
                        let imm = (((instruction >> 7) as u8 & 0b1111) << 4)
                            | (((instruction >> 11) as u8 & 0b11) << 2)
                            | (((instruction >> 5) as u8 & 0b1) << 1)
                            | ((instruction >> 6) as u8 & 0b1);
                        let imm = imm as u16;
                        I {
                            opcode: 0b0010011,
                            rd: rd + RVC_REG_OFFSET,
                            funct3: 0x0,
                            rs1: 0x02,
                            imm: imm.overflowing_mul(4).0 as i16,
                        }
                    }
                    _ => {
                        return Err(InstructionDecodingError);
                    }
                }
            }
            // C1
            0b01 => {
                let funct3 = instruction >> 13;
                match funct3 {
                    // CI-Type: c.nop || c.addi x2, -1
                    0b000 => {
                        // addi x0, x0, 0
                        let rd = ((instruction >> 7) & 0b11111) as u8;
                        let imm = ((((instruction >> 2) & 0b11111) as u8) << 2)
                            | ((((instruction >> 12) & 0b1) as u8) << 7);
                        let imm = (imm as i8) >> 2;
                        I {
                            opcode: 0b0010011,
                            rd,
                            funct3: 0,
                            rs1: rd,
                            imm: imm as i16,
                        }
                    }
                    // CI-Type: c.li -> addi rd, x0, nzimm
                    0b010 => {
                        let rd = ((instruction >> 7) & 0b11111) as u8;
                        //  nzuimm[5|4:0]
                        let imm = (((instruction >> 12) as u8 & 0b1) << 7)
                            | (((instruction >> 2) as u8 & 0b11111) << 2);
                        let imm = (imm as i8) >> 2;
                        I {
                            opcode: 0b0010011,
                            rd,
                            funct3: 0x0,
                            rs1: 0x00,
                            imm: imm as i16,
                        }
                    }
                    // CI-Type: c.lui -> addi rd, x0, nzimm
                    0b011 => {
                        let rd = ((instruction >> 7) & 0b11111) as u8;

                        if rd == 0x2 {
                            // c.addi16sp
                            // nzimm[9] nzimm[4|6|8:7|5]
                            let imm = (((instruction >> 12) as u8 & 0b1) << 7)
                                | (((instruction >> 6) as u8 & 0b1) << 2)
                                | (((instruction >> 5) as u8 & 0b1) << 4)
                                | (((instruction >> 3) as u8 & 0b11) << 5)
                                | (((instruction >> 2) as u8 & 0b1) << 3);
                            let imm = ((imm as i8) as i16) << 2;

                            I {
                                opcode: 0b0010011,
                                rd,
                                funct3: 0x0,
                                rs1: 0x2, // sp
                                imm,
                            }
                        } else {
                            // c.lui
                            //  nzuimm[5|4:0]
                            let imm = (((instruction >> 12) as u8 & 0b1) << 7)
                                | (((instruction >> 2) as u8 & 0b11111) << 2);
                            let imm = (imm as i8) >> 2;

                            U {
                                opcode: 0b0110111,
                                rd,
                                imm: imm as i32,
                            }
                        }
                    }
                    0b100 => {
                        let funct2 = (instruction >> 10) & 0b11;
                        let rd = (instruction >> 7) as u8 & 0b111;
                        // imm/shamt[5] imm/shamt[4:0]
                        let imm = ((instruction >> 5) as u8 & 0b1000_0000)
                            | (instruction as u8 & 0b111_1100);
                        let imm = (imm as i8 >> 2) as i16;

                        // CR-Type: c.srli/c.srai/c.andi
                        match funct2 {
                            // c.srli
                            0b00 => I {
                                opcode: 0b0010011,
                                rd: rd + RVC_REG_OFFSET,
                                funct3: 0x5,
                                rs1: rd + RVC_REG_OFFSET,
                                imm,
                            },
                            // c.srai
                            0b01 => {
                                let imm = ((instruction >> 5) as u8 & 0b1000_0000)
                                    | (instruction as u8 & 0b111_1100);
                                let imm = ((imm >> 2) as u16 & 0b0000_0000_0001_1111) | (0x20 << 5);
                                I {
                                    opcode: 0b0010011,
                                    rd: rd + RVC_REG_OFFSET,
                                    funct3: 0x5,
                                    rs1: rd + RVC_REG_OFFSET,
                                    imm: imm as i16,
                                }
                            }
                            // c.andi
                            0b10 => I {
                                opcode: 0b0010011,
                                rd: rd + RVC_REG_OFFSET,
                                funct3: 0x7,
                                rs1: rd + RVC_REG_OFFSET,
                                imm,
                            },
                            // CS-Type: c.and/c.or/c.xor/c.sub
                            _ => {
                                let funct6 = (instruction >> 10) as u8 & 0b111111;
                                let funct2 = (instruction >> 5) as u8 & 0b11;
                                let rd = (instruction >> 7) as u8 & 0b111;
                                let rs2 = (instruction >> 2) as u8 & 0b111;

                                match (funct6, funct2) {
                                    // c.and
                                    (0b100011, 0b11) => R {
                                        opcode: 0b0110011,
                                        rd: rd + RVC_REG_OFFSET,
                                        funct3: 0x7,
                                        rs1: rd + RVC_REG_OFFSET,
                                        rs2: rs2 + RVC_REG_OFFSET,
                                        funct7: 0x00,
                                    },
                                    // c.or
                                    (0b100011, 0b10) => R {
                                        opcode: 0b0110011,
                                        rd: rd + RVC_REG_OFFSET,
                                        funct3: 0x6,
                                        rs1: rd + RVC_REG_OFFSET,
                                        rs2: rs2 + RVC_REG_OFFSET,
                                        funct7: 0x00,
                                    },
                                    // c.xor
                                    (0b100011, 0b01) => R {
                                        opcode: 0b0110011,
                                        rd: rd + RVC_REG_OFFSET,
                                        funct3: 0x4,
                                        rs1: rd + RVC_REG_OFFSET,
                                        rs2: rs2 + RVC_REG_OFFSET,
                                        funct7: 0x00,
                                    },
                                    // c.sub
                                    (0b100011, 0b00) => R {
                                        opcode: 0b0110011,
                                        rd: rd + RVC_REG_OFFSET,
                                        funct3: 0x0,
                                        rs1: rd + RVC_REG_OFFSET,
                                        rs2: rs2 + RVC_REG_OFFSET,
                                        funct7: 0x20,
                                    },
                                    _ => {
                                        return Err(InstructionDecodingError);
                                    }
                                }
                            }
                        }
                    }
                    // c.j
                    0b101 => {
                        // imm[11|4|9:8|10|6|7|3:1|5]
                        let imm = (((instruction >> 12) & 0b1) << 15)
                            | (((instruction >> 11) & 0b1) << 8)
                            | (((instruction >> 9) & 0b11) << 12)
                            | (((instruction >> 8) & 0b1) << 14)
                            | (((instruction >> 7) & 0b1) << 10)
                            | (((instruction >> 6) & 0b1) << 11)
                            | (((instruction >> 3) & 0b111) << 5)
                            | (((instruction >> 2) & 0b1) << 9);
                        let imm = imm as i16 >> 5;
                        J {
                            opcode: 0b1101111,
                            rd: 0x0,
                            imm: (2 * imm) as i32,
                        }
                    }
                    // c.jal
                    0b001 => {
                        // imm[11|4|9:8|10|6|7|3:1|5]
                        let imm = (((instruction >> 12) & 0b1) << 15)
                            | (((instruction >> 11) & 0b1) << 8)
                            | (((instruction >> 9) & 0b11) << 12)
                            | (((instruction >> 8) & 0b1) << 14)
                            | (((instruction >> 7) & 0b1) << 10)
                            | (((instruction >> 6) & 0b1) << 11)
                            | (((instruction >> 3) & 0b111) << 5)
                            | (((instruction >> 2) & 0b1) << 9);
                        let imm = imm as i16 >> 5;
                        J {
                            opcode: 0b1101111,
                            rd: 0x1,
                            imm: (2 * imm) as i32,
                        }
                    }
                    // c.beqz
                    0b110 => {
                        let rs1 = (instruction >> 7) as u8 & 0b111;
                        // offset[8|4:3] offset[7:6|2:1|5]
                        let imm = (((instruction >> 12) as u8 & 0b1) << 7)
                            | (((instruction >> 10) as u8 & 0b11) << 2)
                            | (((instruction >> 5) as u8 & 0b11) << 5)
                            | ((instruction >> 3) as u8 & 0b11)
                            | (((instruction >> 2) as u8 & 0b1) << 4);
                        let imm = (imm as i8 as i16) << 1;
                        B {
                            opcode: 0b1100011,
                            funct3: 0x0,
                            rs1: rs1 + RVC_REG_OFFSET,
                            rs2: 0x0,
                            imm,
                        }
                    }
                    // c.bnez
                    0b111 => {
                        let rs1 = (instruction >> 7) as u8 & 0b111;
                        // offset[8|4:3] offset[7:6|2:1|5]
                        let imm = (((instruction >> 12) as u8 & 0b1) << 7)
                            | (((instruction >> 10) as u8 & 0b11) << 2)
                            | (((instruction >> 5) as u8 & 0b11) << 5)
                            | ((instruction >> 3) as u8 & 0b11)
                            | (((instruction >> 2) as u8 & 0b1) << 4);
                        let imm = (imm as i8 as i16) << 1;
                        B {
                            opcode: 0b1100011,
                            funct3: 0x1,
                            rs1: rs1 + RVC_REG_OFFSET,
                            rs2: 0x0,
                            imm,
                        }
                    }
                    _ => {
                        return Err(InstructionDecodingError);
                    }
                }
            }
            // C2
            0b10 => {
                let funct4 = instruction >> 12;
                let rs1 = ((instruction >> 7) & 0b11111) as u8;
                let rs2 = ((instruction >> 2) & 0b11111) as u8;
                match funct4 {
                    // CI-Type: c.slli
                    0b0000 | 0b0001 => {
                        // imm/shamt[5] imm/shamt[4:0]
                        let imm = ((instruction >> 5) as u8 & 0b1000_0000)
                            | (instruction as u8 & 0b111_1100);
                        let imm = (imm as i8 >> 2) as i16;
                        I {
                            opcode: 0b0010011,
                            rd: rs1,
                            funct3: 0x1,
                            rs1,
                            imm,
                        }
                    }
                    // CR-Type: c.mv x12, x1 / c.jr
                    0b1000 => {
                        // c.jr
                        if rs1 != 0 && rs2 == 0 {
                            I {
                                opcode: 0b1100111,
                                rd: 0x0, // x0
                                funct3: 0x0,
                                rs1,
                                imm: 0,
                            }
                        } else {
                            // c.mv
                            I {
                                opcode: 0b0010011,
                                rd: rs1,
                                funct3: 0x0,
                                rs1: rs2,
                                imm: 0,
                            }
                        }
                    }
                    // CR-Type: c.add / c.ebreak / c.jalr
                    0b1001 => {
                        if rs1 != 0 && rs2 == 0 {
                            // c.jalr
                            I {
                                opcode: 0b1100111,
                                rd: 0x1, // ra
                                funct3: 0x0,
                                rs1,
                                imm: 0,
                            }
                        } else if rs1 == 0 && rs2 == 0 {
                            // c.ebreak
                            I {
                                opcode: 0b1110011,
                                funct3: 0x0,
                                imm: 0x1,
                                rd: 0,
                                rs1: 0,
                            }
                        } else {
                            // c.add
                            R {
                                opcode: 0b0110011,
                                rd: rs1,
                                funct3: 0x0,
                                funct7: 0x0,
                                rs1,
                                rs2,
                            }
                        }
                    }
                    // CI-Type: c.lwsp x4, 0
                    0b0100 | 0b0101 => {
                        let rs1 = ((instruction >> 7) & 0b11111) as u8;
                        let imm = (((instruction >> 2) as u8 & 0b11) << 6)
                            | (((instruction >> 12) as u8 & 0b1) << 5)
                            | (((instruction >> 4) as u8 & 0b111) << 2);
                        I {
                            opcode: 0b0000011,
                            funct3: 0x2,
                            rd: rs1,
                            rs1: 0x2, // sp
                            imm: imm as i16,
                        }
                    }
                    // CSS-Type: c.swsp x4, 0
                    0b1100 | 0b1101 => {
                        //  uimm[5:2|7:6]
                        let imm = (((instruction >> 9) as u8 & 0b1111) << 2)
                            | (((instruction >> 7) as u8 & 0b11) << 6);
                        S {
                            opcode: 0b0100011,
                            funct3: 0x2,
                            rs1: 0x2, // sp
                            rs2,
                            imm: imm as i16,
                        }
                    }
                    _ => {
                        return Err(InstructionDecodingError);
                    }
                }
            }
            _ => {
                panic!("Instruction should be type C")
            }
        };

        Ok(ins)
    }
}

#[cfg(test)]
mod tests {
    use crate::ins::{Instruction, InstructionFormat};
    use crate::reg::treg;

    #[test]
    fn test_sw_80000130() {
        let ins = Instruction::IRV32(0x0181a023);

        let decoded = ins.decode().expect("decode").1;
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
        let ins = Instruction::IRV32(0x015a8ab3);

        let decoded = ins.decode().expect("decode").1;
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
        let ins = Instruction::IRV32(0xffe00b13);

        let decoded = ins.decode().expect("decode").1;
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
        let ins = Instruction::IRV32(0x17812483);

        let decoded = ins.decode().expect("decode").1;
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
        let ins = Instruction::IRV32(0x0200006f);

        let decoded = ins.decode().expect("decode").1;
        match decoded {
            InstructionFormat::J { opcode, rd, imm } => {
                assert_eq!(opcode, 0b1101111, "opcode wrong");
                assert_eq!(rd, treg("zero"), "rd wrong");
                assert_eq!(imm, 32, "imm wrong");
            }
            _ => assert!(false, "not J"),
        }
    }

    #[test]
    fn test_magic_800032c4() {
        let ins = Instruction::IRV32(0xd41f2023);

        let decoded = ins.decode().expect("decode").1;
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

    #[test]
    fn test_beq_8000093c() {
        let ins = Instruction::IRV32(0x00258593);

        let decoded = ins.decode().expect("decode").1;
        println!("{:032b} {}", 0x00258593, decoded);
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
                assert_eq!(rs1, treg("a1"), "rs1 wrong");
                assert_eq!(rd, treg("a1"), "rd wrong");
                assert_eq!(imm, 2, "imm wrong");
            }
            _ => assert!(false, "not sw"),
        }
    }

    #[test]
    fn test_caddi4spn_80000122() {
        // c.addi4spn x14, 28
        let ins = Instruction::CRV32(0x0878);

        let decoded = ins.decode().expect("decode").1;
        println!("{:032b} {}", 0x0050006f, decoded);
        match decoded {
            InstructionFormat::I {
                opcode,
                funct3,
                rs1,
                imm,
                rd,
            } => {
                assert_eq!(opcode, 0b0010011, "opcode wrong");
                assert_eq!(funct3, 0x0, "funct3 wrong");
                assert_eq!(rd, treg("a4"), "rd wrong");
                assert_eq!(rs1, treg("sp"), "rs1 wrong");
                assert_eq!(imm, 28, "imm wrong");
            }
            _ => assert!(false, "not sw"),
        }
    }

    #[test]
    fn test_cli_80000120() {
        // li	a0,-32
        let ins = Instruction::CRV32(0x5501);

        let decoded = ins.decode().expect("decode").1;
        println!("{:016b} {}", 0x5501, decoded);
        match decoded {
            InstructionFormat::I {
                opcode,
                funct3,
                rs1,
                imm,
                rd,
            } => {
                assert_eq!(opcode, 0b0010011, "opcode wrong");
                assert_eq!(funct3, 0x0, "funct3 wrong");
                assert_eq!(rd, treg("a0"), "rd wrong");
                assert_eq!(rs1, treg("zero"), "rs1 wrong");
                assert_eq!(imm, -32, "imm wrong");
            }
            _ => assert!(false, "not sw"),
        }
    }

    #[test]
    fn test_clw_80000140() {
        // c.lw x12, 48(x12)
        let ins = Instruction::CRV32(0x5a10);

        let decoded = ins.decode().expect("decode").1;
        println!("{:016b} {}", 0x5a10, decoded);
        match decoded {
            InstructionFormat::I {
                opcode,
                funct3,
                rs1,
                imm,
                rd,
            } => {
                assert_eq!(opcode, 0b0000011, "opcode wrong");
                assert_eq!(funct3, 0x2, "funct3 wrong");
                assert_eq!(rd, treg("a2"), "rd wrong");
                assert_eq!(rs1, treg("a2"), "rs1 wrong");
                assert_eq!(imm, 48, "imm wrong");
            }
            _ => assert!(false, "not sw"),
        }
    }

    #[test]
    fn test_clw_800001c0() {
        // lw	a4,4(s1)
        let ins = Instruction::CRV32(0x40d8);

        let decoded = ins.decode().expect("decode").1;
        println!("{:016b} {}", 0x40d8, decoded);
        match decoded {
            InstructionFormat::I {
                opcode,
                funct3,
                rs1,
                imm,
                rd,
            } => {
                assert_eq!(opcode, 0b0000011, "opcode wrong");
                assert_eq!(funct3, 0x2, "funct3 wrong");
                assert_eq!(rd, treg("a4"), "rd wrong");
                assert_eq!(rs1, treg("s1"), "rs1 wrong");
                assert_eq!(imm, 4, "imm wrong");
            }
            _ => assert!(false, "not sw"),
        }
    }

    #[test]
    fn test_clw_800002c0() {
        // lw	s0,64(a1)
        let ins = Instruction::CRV32(0x41a0);

        let decoded = ins.decode().expect("decode").1;
        println!("{:016b} {}", 0x41a0, decoded);
        match decoded {
            InstructionFormat::I {
                opcode,
                funct3,
                rs1,
                imm,
                rd,
            } => {
                assert_eq!(opcode, 0b0000011, "opcode wrong");
                assert_eq!(funct3, 0x2, "funct3 wrong");
                assert_eq!(rd, treg("s0"), "rd wrong");
                assert_eq!(rs1, treg("a1"), "rs1 wrong");
                assert_eq!(imm, 64, "imm wrong");
            }
            _ => assert!(false, "not sw"),
        }
    }
}
