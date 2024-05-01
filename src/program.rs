use nom_span::Spanned;
use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{take_until, take_while},
    character::complete::{anychar, char, multispace0, space0, space1},
    combinator::{cut, eof, map, not, opt, rest},
    sequence::{pair, preceded, terminated},
    Finish, Slice,
};

use crate::{
    error::{AsmError, AsmErrorKind, IResult},
    imm::parse_sym,
    instr::Instr,
    span::{Offset, Span},
};

#[derive(Debug)]
pub struct Program<'i> {
    input: &'i str,
    code: Vec<Instr>,
    sym: HashMap<String, i32>,
}

trait Addresable: Iterator + Sized {
    fn with_address(self) -> impl Iterator<Item = (u32, Self::Item)> {
        self.enumerate()
            .map(|(addr, code)| ((addr << 2) as u32, code))
    }
}

impl<I: Iterator> Addresable for I {}

impl<'s> Program<'s> {
    pub fn resolve(&self, offset: &Offset) -> Result<i32, AsmError<'s>> {
        let start = offset.offset;
        let end = start + offset.len;
        let sym = &self.input[start..end];

        self.sym.get(sym).copied().ok_or_else(|| AsmError {
            span: Spanned::new(self.input, true).slice(start..end),
            kind: AsmErrorKind::UnknownSym,
        })
    }

    pub fn generate(&self) -> Result<Vec<u32>, AsmError<'s>> {
        self.code
            .iter()
            .with_address()
            .map(|(addr, instr)| instr.code(self, addr))
            .collect()
    }

    pub fn dump_code(&self) -> Result<(), AsmError<'s>> {
        let code = self.generate()?;

        for (addr, instr) in code.into_iter().with_address() {
            println!(
                "{:08x}: {:02x} {:02x} {:02x} {:02x}",
                addr,
                (instr >> 24) & 0xff,
                (instr >> 16) & 0xff,
                (instr >> 8) & 0xff,
                instr & 0xff
            );
        }

        Ok(())
    }
}

#[derive(Debug)]
enum Line {
    Instr(Instr),
    Label(String, Option<Instr>),
}

impl Line {
    fn parse(input: Span<'_>) -> IResult<Self> {
        map(
            pair(
                alt((Self::parse_instr, Self::parse_label)),
                opt(preceded(space1, Self::parse_comment)),
            ),
            |(this, _)| this,
        )(input)
    }

    fn parse_instr(input: Span<'_>) -> IResult<Self> {
        map(Instr::parse, Self::Instr)(input)
    }

    fn parse_label(input: Span<'_>) -> IResult<Self> {
        map(
            pair(
                terminated(parse_sym, char(':')),
                opt(preceded(
                    space1,
                    preceded(not(char('#')), cut(Instr::parse)),
                )),
            ),
            |(label, instr)| Self::Label(label.to_string(), instr),
        )(input)
    }

    fn parse_comment(input: Span<'_>) -> IResult<()> {
        map(preceded(char('#'), anychar), |_| ())(input)
    }
}

impl<'s> Program<'s> {
    pub fn parse(input: &'s str) -> Result<Self, AsmError<'s>> {
        let mut program = Self {
            input,
            code: Default::default(),
            sym: Default::default(),
        };

        let input = Span::new(input, true);
        let _ = program.parse_code(input).finish()?;

        Ok(program)
    }

    fn parse_code<'i>(&mut self, input: Span<'i>) -> IResult<'i, ()> {
        let (mut input, _) = multispace0(input)?;
        loop {
            if input.is_empty() {
                return Ok((input, ()));
            }

            let (input_, line) = alt((take_until("\n"), rest))(input)?;

            let (line, _) = space0(line)?;
            if !line.is_empty() {
                let (line, parsed) = Line::parse(line)?;
                match parsed {
                    Line::Instr(instr) => {
                        self.code.push(instr);
                    }
                    Line::Label(label, instr) => {
                        self.sym.insert(label, self.curr_addr() as i32);

                        if let Some(instr) = instr {
                            self.code.push(instr);
                        }
                    }
                }

                let (line, _) = space0(line)?;
                eof(line)?;
            }

            input = take_while(|c| c == '\n')(input_)?.0;
        }
    }

    fn curr_addr(&self) -> u32 {
        (self.code.len() as u32) << 2
    }
}
