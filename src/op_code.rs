use crate::instr::OpKind;
use literify::literify;
use nom::{character::complete::alpha1, combinator::map_opt, IResult};
use phf::phf_map;

impl OpCode {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        map_opt(alpha1, |s| OP_CODE.get(s).copied())(input)
    }
}

macro_rules! mask {
    ($opcode:literal, $f3:literal, $f7:literal) => {
        (($f7 & 0x7f) << 25) | (($f3 & 0x7) << 12) | ($opcode & 0x7f)
    };
    ($opcode:literal, $f3:literal) => {
        (($f3 & 0x7) << 12) | ($opcode & 0x7f)
    };
}

macro_rules! op_code {
    ($($name:ident : $kind:ident $( ($parser:expr))?, $($t:tt),+);+ $(;)?) => {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, Copy)]
        pub enum OpCode {
            $(
                $name,
            )+
        }

        impl OpCode {
            pub fn mask(&self) -> u32 {
                match self {
                    $(
                        Self::$name => mask!($($t),*),
                    )+
                }
            }

            pub fn kind(&self) -> OpKind {
                match self {
                    $(
                        Self::$name => OpKind::$kind,
                    )+
                }
            }
        }

        literify! {
            static OP_CODE: phf::Map<&'static str, OpCode> = phf_map! {
                $(
                    ~($name) => OpCode::$name,
                )+
            };
        }

    };
}

op_code! {
    add     : R, 0x33, 0x0, 0x00;   // ADD
    sub     : R, 0x33, 0x0, 0x20;   // SUB
    xor     : R, 0x33, 0x4, 0x00;   // XOR
    or      : R, 0x33, 0x6, 0x00;   // OR
    and     : R, 0x33, 0x7, 0x00;   // AND
    sll     : R, 0x33, 0x1, 0x00;   // Shift Left Logical
    srl     : R, 0x33, 0x5, 0x00;   // Shift Right Logical
    sra     : R, 0x33, 0x5, 0x20;   // Shift Right Arith
    slt     : R, 0x33, 0x2, 0x00;   // Set Less Than
    sltu    : R, 0x33, 0x3, 0x00;   // Set Less Than (U)
    addi    : I, 0x13, 0x0;         // ADD Immediate
    xori    : I, 0x13, 0x4;         // XOR Immediate
    ori     : I, 0x13, 0x6;         // OR Immediate
    andi    : I, 0x13, 0x7;         // AND Immediate
    slli    : I, 0x13, 0x5;         // Shift Left Logical Imm
    srli    : I, 0x13, 0x5;         // Shift Right Logical Imm
    srai    : I, 0x13, 0x5;         // Shift Right Arith Imm
    slti    : I, 0x13, 0x2;         // Set Less Than Imm
    sltiu   : I, 0x13, 0x3;         // Set Less Than Imm (U)
    // lb      : I, 0x03, 0x0;         // Load Byte
    // lh      : I, 0x03, 0x1;         // Load Half
    // lw      : I, 0x03, 0x2;         // Load Word
    // lbu     : I, 0x03, 0x4;         // Load Byte (U)
    // lhu     : I, 0x03, 0x5;         // Load Half (U)
    beq     : B, 0x63, 0x0;         // Branch ==
}
