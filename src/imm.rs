use std::{
    fmt::{self, Debug, Display},
    rc::Rc,
};

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1, take_while_m_n},
    character::complete::{digit1, hex_digit1, oct_digit1},
    combinator::{map, map_res, peek},
    sequence::preceded,
    IResult,
};

use crate::program::Program;

#[derive(Clone)]
pub enum Imm {
    Val(i32),
    Sym(Rc<String>),
}

impl Display for Imm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Val(val) => Display::fmt(val, f),
            Self::Sym(sym) => Display::fmt(sym, f),
        }
    }
}

impl Debug for Imm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl From<i32> for Imm {
    fn from(val: i32) -> Self {
        Self::Val(val)
    }
}

pub fn parse_sym(input: &str) -> IResult<&str, &str> {
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
    pub fn parse(input: &str) -> IResult<&str, Self> {
        alt((Self::parse_sym, Self::parse_imm))(input)
    }

    pub fn parse_imm(input: &str) -> IResult<&str, Self> {
        map(
            alt((
                Self::parse_decimal,
                Self::parse_hex,
                Self::parse_bin,
                Self::parse_octal,
            )),
            Self::Val,
        )(input)
    }

    pub fn parse_sym(input: &str) -> IResult<&str, Self> {
        map(parse_sym, |sym: &str| Self::Sym(Rc::new(sym.to_string())))(input)
    }

    fn parse_decimal(input: &str) -> IResult<&str, i32> {
        map_res(
            preceded(
                peek(take_while_m_n(1, 1, |c| matches!(c, '1'..='9'))),
                digit1,
            ),
            |s: &str| s.parse::<i32>(),
        )(input)
    }

    fn parse_hex(input: &str) -> IResult<&str, i32> {
        map_res(preceded(tag("0x"), hex_digit1), |s: &str| {
            i32::from_str_radix(s, 16)
        })(input)
    }

    fn parse_bin(input: &str) -> IResult<&str, i32> {
        map_res(preceded(tag("0b"), oct_digit1), |s: &str| {
            i32::from_str_radix(s, 2)
        })(input)
    }

    fn parse_octal(input: &str) -> IResult<&str, i32> {
        map_res(
            preceded(tag("0"), take_while1(|c| matches!(c, '0'..='7'))),
            |s: &str| i32::from_str_radix(s, 8),
        )(input)
    }

    pub fn resolve(&self, program: &Program) -> anyhow::Result<i32> {
        match self {
            Self::Val(val) => Ok(*val),
            Self::Sym(sym) => program.resolve(sym),
        }
    }

    // pub fn mask(&self, program: &Program, shift: u32) -> anyhow::Result<u32> {
    //
    //     (self.0 & 0xfff) << shift
    // }
}
