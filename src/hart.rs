use InstructionFormat::{R, I, S, B, U, J};
use crate::ram::Ram;

pub struct Hart {
    memory: Ram,

    // Registers
    registers: [u32; 32],
    // t6:
    pc: u32,
}

impl Hart {
    pub(crate) fn new(ram: Ram) -> Self {
        let m = Hart {
            memory: ram,
            registers: [0; 32],
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
            1..=31 => self.registers[reg as usize] = val,
            _ => { panic!() }
        }
    }

    fn get_register(&self, reg: u8) -> u32 {
        match reg {
            0..=31 => self.registers[reg as usize],
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
            0b0110011 => {
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
            0b0100011 => {
                println!("{:07b} S-type", opcode);
                let funct3 = ((instruction >> 12) & 0b111) as u8;
                let rs1 = ((instruction >> 15) & 0b1111) as u8;
                let rs2 = ((instruction >> 20) & 0b1111) as u8;
                let imm7 = (instruction >> 7) & 0b11111;
                let imm25 = instruction & 0xfff00000;
                let imm = ((imm25 + (imm7 << 20)) as i32 as u64 >> 20) as i16;
                S { opcode, funct3, rs1, rs2, imm }
            }
            0b1100011 => {
                println!("{:07b} B-type", opcode);
                let funct3 = ((instruction >> 12) & 0b111) as u8;
                let rs1 = ((instruction >> 15) & 0b1111) as u8;
                let rs2 = ((instruction >> 20) & 0b1111) as u8;
                let imm7 = (instruction >> 7) & 0b11111;
                let imm25 = instruction & 0xfff00000;
                let imm = ((imm25 + (imm7 << 20)) as i32 as u64 >> 20) as i16;
                B { opcode, funct3, rs1, rs2, imm }
            }
            0b1101111 => {
                println!("{:07b} J-type", opcode);
                let rd = ((instruction & 0x0F80) >> 7) as u8;
                let imm = ((instruction & 0x7ffff800) as i32 as u64 >> 12) as i32;
                J { opcode, rd, imm }
            }
            0b0110111 | 0b0010111 => {
                println!("{:07b} U-type", opcode);
                let rd = ((instruction >> 7) & 0x1F) as u8;
                let imm = ((instruction & 0x7ffff800) as i32 as u64 >> 12) as i32;
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
            R { opcode: 0b0110011, rd, funct3: 0x0, rs1, rs2, funct7: 0x00 } => {
                let val = self.get_register(rs1).wrapping_add(self.get_register(rs2));
                self.set_register(rd, val)
            }
            // ADD immediate
            I { opcode: 0b0010011, rd, funct3: 0x0, rs1, imm } => {
                let val = self.get_register(rs1).wrapping_add(imm as u32);
                self.set_register(rd, val)
            }
            // lb Load Byte
            I {opcode: 0b0000011, rd, funct3: 0x0, rs1, imm} => {
                let addr = (self.get_register(rs1).wrapping_add(imm as u32)) as usize;
                let val = self.read_byte(addr);
                self.set_register(rd, val as u32)
            }
            // sb Store Byte
            S { opcode: 0b0100011, funct3: 0x00, rs1, rs2, imm } => {
                let addr = (self.get_register(rs1).wrapping_add(imm as u32)) as usize;
                let val = self.get_register(rs2 & 0xF) as u8;
                self.write_byte(addr, val)
            }
            // beq Branch ==
            B { opcode: 0b1100011, funct3: 0x00, rs1, rs2, imm } => {
                if self.get_register(rs1) == self.get_register(rs2) {
                    self.pc += imm as u32
                }
            }
            // jal Jump And Link
            J { opcode: 0b1101111, rd, imm } => {
                self.set_register(rd, self.pc + 4);
                self.pc += imm as u32
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

    fn read_byte(&mut self, addr: usize) -> u8 {
        self.memory.read_byte(addr)
    }
    fn read_word(&mut self, addr: usize) -> u32 {
        self.memory.read_word(addr)
    }

    fn write_byte(&mut self, addr: usize, val: u8) {
        self.memory.write_byte(addr, val)
    }
    fn write_word(&mut self, addr: usize, val: u32) {
        self.memory.write_word(addr, val)
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
    use crate::Hart;
    use crate::ram::Ram;

    #[test]
    fn addi() {
        let ram = Ram::new(vec![0x13, 0x81, 0x00, 0x7d]);
        let mut m = Hart::new(ram);
        m.tick();
        assert_eq!(m.get_register(2), 2000, "x1 mismatch");
    }

    #[test]
    fn addi_neg() {
        let ram = Ram::new(vec![0x93, 0x01, 0x81, 0xc1]);
        let mut m = Hart::new(ram);
        m.tick();
        assert_eq!(m.get_register(3) as i32, -1000, "x1 mismatch");
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
        let mut m = Hart::new(ram);
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
