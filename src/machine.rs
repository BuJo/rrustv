use InstructionFormat::{R, I, S, B, U, J};
use crate::ram::Ram;

pub struct Machine {
    memory: Ram,

    // Registers
    x0: u32,
    // zero : Hard-wired zero
    x1: u32,
    // ra : Return address
    x2: u32,
    // sp: Stack Pointer
    x3: u32,
    // gp: Global Pointer
    x4: u32,
    // tp: Thread Pointer
    x5: u32,
    // t0: Temporaries
    x6: u32,
    // t1: Temporaries
    x7: u32,
    // t2: Temporaries
    x8: u32,
    // s0/fp: Saved Register/Frame Pointer
    x9: u32,
    // s1: Saved Register
    x10: u32,
    // a0: Function arguments/return values
    x11: u32,
    // a1:
    x12: u32,
    // a2: Function arguments
    x13: u32,
    // a3:
    x14: u32,
    // a4:
    x15: u32,
    // a5:
    x16: u32,
    // a6:
    x17: u32,
    // a7:
    x18: u32,
    // s2: Saved registers
    x19: u32,
    // s3:
    x20: u32,
    // s4:
    x21: u32,
    // s5:
    x22: u32,
    // s6:
    x23: u32,
    // s7:
    x24: u32,
    // s8:
    x25: u32,
    // s9:
    x26: u32,
    // s10:
    x27: u32,
    // s11:
    x28: u32,
    // t3: Temporaries
    x29: u32,
    // t4:
    x30: u32,
    // t5:
    x31: u32,
    // t6:
    pc: u32,
}

impl Machine {
    pub(crate) fn new(ram: Ram) -> Self {
        let m = Machine {
            memory: ram,
            x0: 0,
            x1: 0,
            x2: 0,
            x3: 0,
            x4: 0,
            x5: 0,
            x6: 0,
            x7: 0,
            x8: 0,
            x9: 0,
            x10: 0,
            x11: 0,
            x12: 0,
            x13: 0,
            x14: 0,
            x15: 0,
            x16: 0,
            x17: 0,
            x18: 0,
            x19: 0,
            x20: 0,
            x21: 0,
            x22: 0,
            x23: 0,
            x24: 0,
            x25: 0,
            x26: 0,
            x27: 0,
            x28: 0,
            x29: 0,
            x30: 0,
            x31: 0,
            pc: 0,
        };
        m
    }

    pub(crate) fn tick(&mut self) {
        let instruction = self.fetch_instruction();
        let instruction = self.decode_instruction(instruction);
        self.execute_instruction(instruction);
    }

    fn set_register(&mut self, reg: u8, val: u32) {
        match reg {
            0 => { panic!() }
            1 => self.x1 = val,
            2 => self.x2 = val,
            3 => self.x3 = val,
            4 => self.x4 = val,
            5 => self.x5 = val,
            6 => self.x6 = val,
            7 => self.x7 = val,
            8 => self.x8 = val,
            9 => self.x9 = val,
            10 => self.x10 = val,
            11 => self.x11 = val,
            12 => self.x12 = val,
            13 => self.x13 = val,
            14 => self.x14 = val,
            15 => self.x15 = val,
            16 => self.x16 = val,
            17 => self.x17 = val,
            18 => self.x18 = val,
            19 => self.x19 = val,
            20 => self.x20 = val,
            21 => self.x21 = val,
            22 => self.x22 = val,
            23 => self.x23 = val,
            24 => self.x24 = val,
            25 => self.x25 = val,
            26 => self.x26 = val,
            27 => self.x27 = val,
            28 => self.x28 = val,
            29 => self.x29 = val,
            30 => self.x30 = val,
            31 => self.x31 = val,
            _ => { panic!() }
        }
    }

    fn get_register(&self, reg: u8) -> u32 {
        match reg {
            0 => self.x0,
            1 => self.x1,
            2 => self.x2,
            3 => self.x3,
            4 => self.x4,
            5 => self.x5,
            6 => self.x6,
            7 => self.x7,
            8 => self.x8,
            9 => self.x9,
            10 => self.x10,
            11 => self.x11,
            12 => self.x12,
            13 => self.x13,
            14 => self.x14,
            15 => self.x15,
            16 => self.x16,
            17 => self.x17,
            18 => self.x18,
            19 => self.x19,
            20 => self.x20,
            21 => self.x21,
            22 => self.x22,
            23 => self.x23,
            24 => self.x24,
            25 => self.x25,
            26 => self.x26,
            27 => self.x27,
            28 => self.x28,
            29 => self.x29,
            30 => self.x30,
            31 => self.x31,
            _ => { panic!() }
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
            0b0110011 | 0b0100011 => {
                println!("{:07b} R-type", opcode);
                let rd = ((instruction >> 7) & 0b11111) as u8;
                let funct3 = ((instruction >> 12) & 0b111) as u8;
                let rs1 = ((instruction >> 15) & 0b1111) as u8;
                let rs2 = ((instruction >> 20) & 0b1111) as u8;
                let funct7 = (instruction >> 25) as u8;
                R { opcode, rd, funct3, rs1, rs2, funct7 }
            }
            0b0010011 | 0b0000011 | 0b1100111 | 0b1110011 => {
                println!("{:07b} I-type", opcode);
                let rd = ((instruction & 0x0F80) >> 7) as u8;
                let funct3 = ((instruction & 0x7000) >> 12) as u8;
                let rs1 = ((instruction & 0xF8000) >> 15) as u8;
                let imm = ((instruction & 0xfff00000) as i32 as u64 >> 20) as i16;
                I { opcode, rd, funct3, rs1, imm }
            }
            0b1100011 => {
                println!("{:07b} B-type", opcode);
                let funct3 = ((instruction >> 12) & 0b111) as u8;
                let rs1 = ((instruction >> 15) & 0b1111) as u8;
                let rs2 = ((instruction >> 20) & 0b1111) as u8;
                let imm7 = ((instruction >> 7) & 0b11111);
                let imm25 = (instruction & 0xfff00000);
                let imm = ((imm25 + (imm7 << 20)) as i32 as u64 >> 20) as i16;
                B { opcode, funct3, rs1, rs2, imm }
            }
            0b1101111 => {
                println!("{:07b} J-type", opcode);
                todo!();
            }
            0b0110111 | 0b0010111 => {
                println!("{:07b} U-type", opcode);
                let rd = ((instruction >> 7) & 0x1F) as u8;
                let imm = (instruction >> 12) as i32;
                U { opcode, rd, imm }
            }
            _ => {
                println!("{:07b} Unknown opcode", opcode);
                panic!();
            }
        }
    }


    fn execute_instruction(&mut self, instruction: InstructionFormat) {
        println!("{:?}", instruction);

        match instruction {
            // RV32I

            // ADD
            R { opcode: 0b0110011, rd, funct3: 0x00, rs1, rs2, funct7: 0x00 } => {
                let val = self.get_register(rs1).wrapping_add(self.get_register(rs2));
                self.set_register(rd, val)
            }
            // ADD immediate
            I { opcode: 0b0010011, rd, funct3: 0x00, rs1, imm } => {
                let val = self.get_register(rs1).wrapping_add(imm as u32);
                self.set_register(rd, val)
            }
            // beq Branch ==
            B { opcode: 0b1100011, funct3: 0x00, rs1, rs2, imm} => {
                if self.get_register(rs1) ==  self.get_register(rs1) {
                    self.pc += imm
                }
            }
            // auipc Add Upper Imm to PC
            U { opcode: 0b0010111, rd, imm } => {
                let val = self.pc + ((imm as u32) << 12);
                self.set_register(rd, val)
            }

            _ => {
                println!("Unknown instruction: {:?}", instruction);
                todo!()
            }
        }
    }
}

#[derive(Debug)]
enum InstructionFormat {
    R { opcode: u8, rd: u8, funct3: u8, rs1: u8, rs2: u8, funct7: u8 },
    I { opcode: u8, rd: u8, funct3: u8, rs1: u8, imm: i16 },
    S { opcode: u8, funct3: u8, rs1: u8, rs2: u8, imm: i16 },
    B { opcode: u8, funct3: u8, rs1: u8, rs2: u8, imm: i16 },
    U { opcode: u8, rd: u8, imm: i32 },
    J { opcode: u8, rd: u8, imm: i32 },
}


#[cfg(test)]
mod tests {
    use crate::Machine;
    use crate::ram::Ram;

    #[test]
    fn addi() {
        let ram = Ram::new(vec![0x13, 0x81, 0x00, 0x7d]);
        let mut m = Machine::new(ram);
        m.tick();
        assert_eq!(m.x2, 2000, "x1 mismatch");
    }

    #[test]
    fn addi_neg() {
        let ram = Ram::new(vec![0x93, 0x01, 0x81, 0xc1]);
        let mut m = Machine::new(ram);
        m.tick();
        assert_eq!(m.x3 as i32, -1000, "x1 mismatch");
    }

    #[test]
    fn it_works() {
        let ram = Ram::new(vec![
            // li	ra,1000
            0x93, 0x00, 0x80, 0x3e,
            // addi	sp,ra,2000
            0x13, 0x81, 0x00, 0x7d,
            // addi	gp,sp,-1000
            0x93, 0x01, 0x81, 0xc1,
            // addi	tp,gp,-2000
            0x13, 0x82, 0x01, 0x83,
            // addi	t0,tp,1000
            0x93, 0x02, 0x82, 0x3e,
            // li	t1,64
            0x13, 0x03, 0x00, 0x04,
            // addi	t1,t1,4
            0x13, 0x03, 0x43, 0x00,
        ]);
        let mut m = Machine::new(ram);
        m.tick();
        m.tick();
        m.tick();
        m.tick();
        m.tick();
        m.tick();
        m.tick();
        assert_eq!(m.x0, 0, "zero register must be zero");
        assert_eq!(m.x1, 1000, "x1 mismatch");
        assert_eq!(m.x2, 3000, "x2 mismatch");
        assert_eq!(m.x3, 2000, "x3 mismatch");
        assert_eq!(m.x4, 0, "x4 mismatch");
        assert_eq!(m.x5, 1000, "x5 mismatch");
        assert_eq!(m.x6, 0x40 + 4, "deadbeef");
    }
}
