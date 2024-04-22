use nom::{
    branch::alt,
    character::complete::{char, space0, space1},
    combinator::map,
    sequence::{delimited, terminated},
    IResult,
};

use crate::{
    imm::Imm,
    op_code::{OpCode, OpCodeTy},
    pseudo::Pseudo,
    reg::Reg,
};

macro_rules! parse_operands {
    ($input:ident as $name0:ident => $parser0:expr $(,$name:ident => $parser:expr)* $(,)?) => {
        let ($input, $name0) = $parser0($input)?;
        $(
            let ($input, _) = delimited(space0, char(','), space0)($input)?;
            let ($input, $name) = $parser($input)?;
        )*
    };
}

#[derive(Debug)]
pub struct Instr {
    pub op_code: OpCode,
    pub operands: Operands,
}

impl Instr {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, this) = alt((Self::parse_pseudo, Self::parse_instr))(input)?;
        let (input, _) = space0(input)?;

        Ok((input, this))
    }

    fn parse_pseudo(input: &str) -> IResult<&str, Self> {
        let (input, pseudo) = terminated(Pseudo::parse, space1)(input)?;

        let op_code = pseudo.op_code();
        match pseudo {
            Pseudo::mv => {
                assert_eq!(op_code.ty(), OpCodeTy::I);

                map(Self::parse_pseudo_rd_rs, |(rd, rs)| Self {
                    op_code,
                    operands: Operands::I(InstrI {
                        rd,
                        rs,
                        imm: 0.into(),
                    }),
                })(input)
            }
        }
    }

    fn parse_pseudo_rd_rs(input: &str) -> IResult<&str, (Reg, Reg)> {
        parse_operands! {
            input as
            rd => Reg::parse,
            rs => Reg::parse
        };

        Ok((input, (rd, rs)))
    }

    fn parse_instr(input: &str) -> IResult<&str, Self> {
        let (input, op_code) = terminated(OpCode::parse, space1)(input)?;

        let (input, operands) = (match op_code.ty() {
            OpCodeTy::R => map(InstrR::parse, Operands::R)(input),
            OpCodeTy::I => map(InstrI::parse, Operands::I)(input),
        })?;

        Ok((input, Self { op_code, operands }))
    }

    pub fn code(&self) -> u32 {
        (self.op_code as u32) | self.operands.code()
    }

    pub fn to_hex_str(&self) -> String {
        format!("{:08X}", self.code())
    }
}

#[derive(Debug)]
pub enum Operands {
    R(InstrR),
    I(InstrI),
}

impl Operands {
    pub fn code(&self) -> u32 {
        match self {
            Self::R(instr) => instr.code(),
            Self::I(instr) => instr.code(),
        }
    }
}

#[derive(Debug)]
pub struct InstrR {
    pub rd: Reg,
    pub rs1: Reg,
    pub rs2: Reg,
}

impl InstrR {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        parse_operands! {
            input as
            rd => Reg::parse,
            rs1 => Reg::parse,
            rs2 => Reg::parse
        };

        Ok((input, Self { rd, rs1, rs2 }))
    }

    pub fn code(&self) -> u32 {
        self.rs2.code(20) | self.rs1.code(15) | self.rd.code(7)
    }
}

#[derive(Debug)]
pub struct InstrI {
    pub rd: Reg,
    pub rs: Reg,
    pub imm: Imm,
}

impl InstrI {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        parse_operands! {
            input as
            rd => Reg::parse,
            rs => Reg::parse,
            imm => Imm::parse
        };

        Ok((input, Self { rd, rs, imm }))
    }

    pub fn code(&self) -> u32 {
        self.imm.code(20) | self.rs.code(15) | self.rd.code(7)
    }
}
