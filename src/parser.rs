use std::fmt::{self, Display};

use nom::{
    branch::alt,
    bytes::complete::{take_until, take_while},
    character::complete::{alpha1, alphanumeric1, char, multispace0, space0, space1},
    combinator::{map_opt, rest},
    error::{Error, ErrorKind},
    sequence::delimited,
    Err, Finish, IResult,
};

use crate::{
    op_code::{OpCode, OpCodeTy, INSTR},
    reg::{Reg, REGS},
};

#[derive(Default)]
pub struct Program(Vec<u32>);

impl Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (addr, code) in self.0.iter().enumerate() {
            writeln!(
                f,
                "{:08x}:  {:02x} {:02x} {:02x} {:02x}",
                addr << 2,
                (code >> 24) & 0xff,
                (code >> 16) & 0xff,
                (code >> 8) & 0xff,
                code & 0xff
            )?;
        }

        Ok(())
    }
}

impl Program {
    pub fn parse(input: &str) -> Result<Self, Error<&str>> {
        Self::parse_(input).finish().map(|(_, program)| program)
    }

    fn parse_(input: &str) -> IResult<&str, Self> {
        let mut program = vec![];

        let (mut input, _) = multispace0(input)?;
        loop {
            if input.is_empty() {
                return Ok((input, Self(program)));
            }

            let (input_, line) = alt((take_until("\n"), rest))(input)?;

            let (line, _) = space0(line)?;
            if !line.is_empty() {
                let (line, instr) = Instr::parse(line)?;
                if !line.is_empty() {
                    return Err(Err::Error(Error::new(line, ErrorKind::NonEmpty)));
                }
                program.push(instr.code());
            }

            input = take_while(|c| c == '\n')(input_)?.0;
        }
    }
}

#[derive(Debug)]
pub struct Instr {
    pub op_code: OpCode,
    pub operands: Operands,
}

impl Instr {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        let (input, op_code) = op_code(input)?;
        let (input, operands) = match op_code.ty() {
            OpCodeTy::R => {
                let (input, instr_r) = InstrR::parse(input)?;
                (input, Operands::R(instr_r))
            }
        };

        Ok((input, Instr { op_code, operands }))
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
}

impl Operands {
    pub fn code(&self) -> u32 {
        match self {
            Self::R(instr) => instr.code(),
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
        let (input, rd) = delimited(space1, reg, space0)(input)?;
        let (input, _) = char(',')(input)?;
        let (input, rs1) = delimited(space0, reg, space0)(input)?;
        let (input, _) = char(',')(input)?;
        let (input, rs2) = delimited(space0, reg, space0)(input)?;

        Ok((input, InstrR { rd, rs1, rs2 }))
    }

    pub fn code(&self) -> u32 {
        self.rs2.code(20) | self.rs1.code(15) | self.rd.code(7)
    }
}

pub fn op_code(input: &str) -> IResult<&str, OpCode> {
    map_opt(alpha1, |s| INSTR.get(s).copied())(input)
}

pub fn reg(input: &str) -> IResult<&str, Reg> {
    map_opt(alphanumeric1, |s| REGS.get(s).copied().map(Into::into))(input)
}
