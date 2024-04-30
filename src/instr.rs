use nom::{
    branch::alt,
    character::complete::{char, space0, space1},
    combinator::{cut, map},
    error::context,
    sequence::{delimited, terminated},
};
use std::fmt::Debug;

use crate::{
    imm::Imm,
    op_code::OpCode,
    program::Program,
    pseudo::Pseudo,
    reg::Reg,
    span::{IResult, Span},
};

macro_rules! parse_ops {
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
    pub fn parse(input: Span<'_>) -> IResult<Self> {
        let (input, this) = alt((Self::parse_pseudo, Self::parse_instr))(input)?;
        let (input, _) = space0(input)?;

        Ok((input, this))
    }

    fn parse_pseudo(input: Span<'_>) -> IResult<Self> {
        let (input, pseudo) = terminated(Pseudo::parse, space1)(input)?;

        let op_code = pseudo.op_code();
        cut(move |input| match pseudo {
            Pseudo::mv => {
                assert_eq!(op_code.kind(), OpKind::I);

                map(Self::parse_pseudo_rd_rs, |(rd, rs)| Self {
                    op_code,
                    operands: Operands::I(InstrI {
                        rd,
                        rs,
                        imm: 0.into(),
                    }),
                })(input)
            }
        })(input)
    }

    fn parse_pseudo_rd_rs(input: Span<'_>) -> IResult<(Reg, Reg)> {
        parse_ops! {
            input as
            rd => Reg::parse,
            rs => Reg::parse
        };

        Ok((input, (rd, rs)))
    }

    fn parse_instr(input: Span<'_>) -> IResult<Self> {
        let (input, op_code) = terminated(OpCode::parse, space1)(input)?;
        let (input, operands) = cut(|input| op_code.kind().parse(input))(input)?;

        Ok((input, Self { op_code, operands }))
    }

    pub fn code(&self, program: &Program, addr: u32) -> anyhow::Result<u32> {
        let op_code = self.op_code.mask();
        let operands = self.operands.mask(program, addr)?;

        Ok(op_code | operands)
    }
}

pub trait Mask {
    fn mask(&self, program: &Program, addr: u32) -> anyhow::Result<u32>;
}

macro_rules! op_kind {
    ($name:ident with $($field:ident:$ty:ty => $parser:expr),+) => {
        #[derive(Debug)]
        pub struct $name {
            $(
                $field: $ty,
            )+
        }

        impl $name {
            pub fn parse(input: Span<'_>) -> IResult<Self> {
                parse_ops!(
                    input as
                    $(
                        $field => $parser,
                    )+
                );

                Ok((input, Self {
                    $( $field ),+
                }))
            }
        }

    }
}

macro_rules! ops {
    ($($kind:ident => ($name:ident with $($field:ident:$ty:ty => $parser:expr),+)),+) => {
        #[derive(Debug)]
        pub enum Operands {
            $(
                $kind($name),
            )+
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum OpKind {
            $(
                $kind,
            )+
        }

        impl OpKind {
            pub fn parse<'i>(&self, input: Span<'i>) -> IResult<'i, Operands> {
                match self {
                    $(
                        Self::$kind => map(context(stringify!(Instr $kind), $name::parse), Operands::$kind)(input),
                    )+
                }
            }
        }

        impl Mask for Operands {
            fn mask(&self, program: &Program, addr: u32) -> anyhow::Result<u32> {
                match self {
                    $(
                        Self::$kind(instr) => instr.mask(program, addr),
                    )+
                }
            }
        }

        $(
            op_kind!($name with $($field:$ty => $parser),+);
        )+
    };
}

ops! {
    R => (InstrR with
        rd: Reg => Reg::parse,
        rs1: Reg => Reg::parse,
        rs2: Reg => Reg::parse
    ),
    I => (InstrI with
        rd: Reg => Reg::parse,
        rs: Reg => Reg::parse,
        imm: Imm => Imm::parse
    ),
    S => (InstrS with
        rs1: Reg => Reg::parse,
        rs2: Reg => Reg::parse,
        imm: Imm => Imm::parse
    ),
    B => (InstrB with
        rs1: Reg => Reg::parse,
        rs2: Reg => Reg::parse,
        imm: Imm => Imm::parse
    )
}

pub fn slice(imm: u32, start: u32, end: u32) -> u32 {
    let (start, end) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };

    let len = end - start + 1;
    (imm >> start) & ((1 << len) - 1)
}

#[inline]
pub fn bit(imm: u32, idx: u32) -> u32 {
    slice(imm, idx, idx)
}

impl Mask for InstrR {
    fn mask(&self, _: &Program, _: u32) -> anyhow::Result<u32> {
        Ok((self.rs1.idx() << 20) | (self.rs2.idx() << 15) | (self.rd.idx() << 7))
    }
}

impl Mask for InstrI {
    fn mask(&self, program: &Program, _: u32) -> anyhow::Result<u32> {
        let imm = (self.imm.resolve(program)? as u32) & 0xfff;
        let rs = self.rs.idx();
        let rd = self.rd.idx();

        Ok((imm << 20) | (rs << 15) | (rd << 7))
    }
}

impl Mask for InstrS {
    fn mask(&self, program: &Program, _: u32) -> anyhow::Result<u32> {
        let imm = (self.imm.resolve(program)? as u32) & 0xfff;

        Ok((slice(imm, 5, 11) << 25)
            | (self.rs2.idx() << 20)
            | (self.rs1.idx() << 15)
            | (slice(imm, 0, 4) << 7))
    }
}

impl Mask for InstrB {
    fn mask(&self, program: &Program, addr: u32) -> anyhow::Result<u32> {
        let imm = self.imm.resolve(program)? - (addr as i32);
        let imm = (imm as u32) & 0x1fff;

        Ok((bit(imm, 12) << 31)
            | (slice(imm, 5, 10) << 25)
            | (self.rs2.idx() << 20)
            | (self.rs1.idx() << 15)
            | (slice(imm, 1, 4) << 8)
            | (bit(imm, 11) << 7))
    }
}
