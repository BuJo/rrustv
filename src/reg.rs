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

pub fn reg(reg: u8) -> &'static str {
    for (i, s) in REGMAP {
        if i == reg {
            return s;
        }
    }
    "U"
}

pub fn treg(reg: &str) -> u8 {
    for (i, s) in REGMAP {
        if s == reg {
            return i;
        }
    }
    255
}