use std::process::id;
use crate::InstructionFormat::{I, R, S, B, U, J};

struct Machine {
    memory: Vec<u8>,

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
    fn new() -> Self {
        let mut m = Machine {
            memory: vec![0; 128],
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

    fn tick(&mut self) {
        let instruction = self.fetch_instruction();
        let instruction = self.decode_instruction(instruction);
        println!("{:?}", instruction);
        match instruction {
            R { .. } => {}
            I { rd, funct3, rs1, imm, .. } => {
                match funct3 {
                    0x0 => {
                        // ADD immediate
                        self.set_register(rd, self.get_register(rs1) + imm as u32)
                    }
                    _ => { todo!() }
                }
            }
            S { .. } => { todo!() }
            B { .. } => { todo!() }
            U { opcode, rd, imm } => {
                match opcode {
                    0b0010111 => {
                        // auipc Add Upper Imm to PC
                        self.set_register(rd, self.pc + ((imm as u32) << 12))
                    }
                    _ => { todo!() }
                }
            }
            J { .. } => { todo!() }
        }
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

    fn write_word(&mut self, idx: usize, word: u32) {
        self.memory[idx + 0] = ((word >> 0) & 0xFF) as u8;
        self.memory[idx + 1] = ((word >> 8) & 0xFF) as u8;
        self.memory[idx + 2] = ((word >> 16) & 0xFF) as u8;
        self.memory[idx + 3] = ((word >> 24) & 0xFF) as u8;
        //println!("{:?}", self.memory);
    }

    fn fetch_instruction(&mut self) -> u32 {
        let ins: u32 =
            0 +
                ((self.memory[(self.pc + 0) as usize] as u32) << 0) +
                ((self.memory[(self.pc + 1) as usize] as u32) << 8) +
                ((self.memory[(self.pc + 2) as usize] as u32) << 16) +
                ((self.memory[(self.pc + 3) as usize] as u32) << 24);
        //println!("{:#x} ", ins);
        self.pc += 4;
        ins
    }
    fn decode_instruction(&self, instruction: u32) -> InstructionFormat {
        let opcode = (instruction & 0b1111111) as u8;
        match opcode {
            0b0110011 => {
                println!("R-type");
                todo!();
            }
            0b0010011 | 0b0000011 => {
                println!("I-type");
                let rd = ((instruction & 0x0F80) >> 7) as u8;
                let funct3 = ((instruction & 0x7000) >> 12) as u8;
                let rs1 = ((instruction & 0xF8000) >> 15) as u8;
                let signed = (instruction >> 31) > 0;
                println!("original: {}", instruction);
                println!("signed: {}", ((instruction >> 20) & 0x800) > 0);
                println!("value: {}", ((instruction >> 20) & 0x7FF));
                println!("value: {}", ((instruction >> 20) as i16));
                println!("value: {:b}", (instruction >> 20));
                println!("value: {:b}", ((instruction >> 20) & 0x7FF));
                let bits = ((instruction >> 20) & 0x7FF) as u16;
                let imm = if signed {
                    println!("-1: {:b}", (-1i16 as u16));
                    println!("&: {:b}", (-1i16 * bits as i16));
                    println!("&: {:b}", (0xFFFF & bits));
                    println!("&: {:b}", (0xF000 | bits));
                    println!("&: {:b}", (0xF000 | bits));
                    println!("&: {:b}", (-1i16 * bits as i16) as u16);
                    (0xF000 | bits) as i16
                } else {
                    bits as i16
                };
                println!("imm: {}: {:#x}, {:012b}", imm, imm, imm);
                let ins = I { opcode, rd, funct3, rs1, imm };
                ins
            }
            0b0100011 => {
                println!("R-type");
                todo!();
            }
            0b1100011 => {
                println!("B-type");
                todo!();
            }
            0b1101111 => {
                println!("J-type");
                todo!();
            }
            0b1100111 => {
                println!("I-type");
                todo!();
            }
            0b0110111 => {
                println!("U-type");
                todo!();
            }
            0b0010111 => {
                println!("U-type");
                let rd = ((instruction >> 7) & 0x1F) as u8;
                let imm = (instruction >> 12) as i32;
                U { opcode, rd, imm }
            }
            0b1110011 => {
                println!("I-type");
                todo!();
            }
            _ => {
                panic!();
            }
        }
    }
}

type Register = u32;

#[derive(Debug)]
enum InstructionFormat {
    R { opcode: u8, rd: u8, funct3: u8, rs1: u8, rs2: u8, funct7: u8 },
    I { opcode: u8, rd: u8, funct3: u8, rs1: u8, imm: i16 },
    S { opcode: u8, funct3: u8, rs1: u8, rs2: u8, imm: i16 },
    B { opcode: u8, funct3: u8, rs1: u8, rs2: u8, imm: i16 },
    U { opcode: u8, rd: u8, imm: i32 },
    J { opcode: u8, rd: u8, imm: i32 },
}


fn main() {
    let mut m = Machine::new();
    m.write_word(0x0000, 0x3e800093);
    m.write_word(0x0004, 0x7d008113);
    m.write_word(0x0008, 0xc1810193);
    m.write_word(0x000c, 0x83018213);
    m.write_word(0x0010, 0x3e820293);
    m.write_word(0x0014, 0x00010317);
    m.write_word(0x0018, 0xfec30313);
    m.write_word(0x001c, 0x00430313);
    m.write_word(0x007f, 0xdeadbeef);
    m.tick();
    m.tick();
    m.tick();
    m.tick();
    m.tick();
    m.tick();
    m.tick();
    m.tick();
}


#[cfg(test)]
mod tests {
    use crate::Machine;

    #[test]
    fn it_works() {
        let mut m = Machine::new();
        m.write_word(0x00, 0x3e800093);
        m.write_word(0x04, 0x7d008113);
        m.write_word(0x08, 0xc1810193);
        m.write_word(0x0c, 0x83018213);
        m.write_word(0x10, 0x3e820293);
        m.write_word(0x14, 0x00010317);
        m.write_word(0x18, 0xfec30313);
        m.write_word(0x1c, 0x00430313);
        m.write_word(0x40, 0xdeadbeef);
        m.tick();
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
        assert_eq!(m.x5, 3735928563, "deadbeef");
    }
}
