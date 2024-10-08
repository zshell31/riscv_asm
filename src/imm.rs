use std::fmt::Debug;

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1, take_while_m_n},
    character::complete::{digit1, hex_digit1, oct_digit1},
    combinator::{map, map_res, peek},
    sequence::preceded,
};

use crate::{
    error::{AsmError, AsmErrorKind, IResult},
    program::Program,
    span::{Offset, Span},
};

#[derive(Debug, Clone, Copy)]
pub enum Imm {
    Val(i32),
    Sym(Offset),
}

impl From<i32> for Imm {
    fn from(val: i32) -> Self {
        Self::Val(val)
    }
}

pub fn parse_sym(input: Span<'_>) -> IResult<Span<'_>> {
    preceded(
        peek(take_while_m_n(
            1,
            1,
            |c| matches!(c, 'a'..='z' | 'A'..='Z' | '_'),
        )),
        take_while1(|c| matches!(c, '0'..='9' | 'a'..='z' | 'A'..='Z' | '_')),
    )(input)
}

impl Imm {
    pub fn parse(input: Span<'_>) -> IResult<Self> {
        alt((Self::parse_sym, Self::parse_imm))(input)
    }

    pub fn parse_imm(input: Span<'_>) -> IResult<Self> {
        map(
            alt((
                Self::parse_decimal,
                Self::parse_hex,
                Self::parse_bin,
                Self::parse_octal,
            )),
            Self::Val,
        )(input)
        .map_err(|e| e.map(|e: AsmError<'_>| e.with_kind(AsmErrorKind::InvalidImm)))
    }

    pub fn parse_sym(input: Span<'_>) -> IResult<Self> {
        map(parse_sym, |sym| Self::Sym(sym.into()))(input)
    }

    fn parse_decimal(input: Span<'_>) -> IResult<i32> {
        map_res(
            preceded(
                peek(take_while_m_n(1, 1, |c| matches!(c, '1'..='9'))),
                digit1,
            ),
            |s: Span<'_>| s.parse::<i32>(),
        )(input)
    }

    fn parse_hex(input: Span<'_>) -> IResult<i32> {
        map_res(preceded(tag("0x"), hex_digit1), |s: Span<'_>| {
            i32::from_str_radix(*s, 16)
        })(input)
    }

    fn parse_bin(input: Span<'_>) -> IResult<i32> {
        map_res(preceded(tag("0b"), oct_digit1), |s: Span<'_>| {
            i32::from_str_radix(*s, 2)
        })(input)
    }

    fn parse_octal(input: Span<'_>) -> IResult<i32> {
        map_res(
            preceded(tag("0"), take_while1(|c| matches!(c, '0'..='7'))),
            |s: Span<'_>| i32::from_str_radix(*s, 8),
        )(input)
    }

    pub fn resolve<'s>(&self, program: &Program<'s>) -> Result<i32, AsmError<'s>> {
        match self {
            Self::Val(val) => Ok(*val),
            Self::Sym(sym) => program.resolve(sym),
        }
    }
}
