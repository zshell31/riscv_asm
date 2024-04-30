use anyhow::anyhow;
use std::collections::HashMap;
use std::fmt::Write;

use nom::{
    branch::alt,
    bytes::complete::{take_until, take_while},
    character::complete::{anychar, char, multispace0, space0, space1},
    combinator::{cut, eof, map, not, opt, rest},
    error::{VerboseError, VerboseErrorKind},
    sequence::{pair, preceded, terminated},
    Finish,
};

use crate::{
    imm::parse_sym,
    instr::Instr,
    span::{IResult, Offset, Span},
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

impl Program<'_> {
    pub fn resolve(&self, offset: &Offset) -> anyhow::Result<i32> {
        let start = offset.offset;
        let end = start + offset.len;
        let sym = &self.input[start..end];

        self.sym
            .get(sym)
            .copied()
            .ok_or_else(|| anyhow!("Cannot find symbol {sym}"))
    }

    pub fn generate(&self) -> anyhow::Result<Vec<u32>> {
        self.code
            .iter()
            .with_address()
            .map(|(addr, instr)| instr.code(self, addr))
            .collect()
    }

    pub fn dump_code(&self) -> anyhow::Result<()> {
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
    pub fn parse(input: &'s str) -> anyhow::Result<Self> {
        let mut program = Self {
            input,
            code: Default::default(),
            sym: Default::default(),
        };

        let input = Span::new(input, true);
        match program.parse_code(input).finish() {
            Ok(_) => Ok(program),
            Err(e) => Err(anyhow!(program.convert_error(e))),
        }
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

    fn convert_error(&self, e: VerboseError<Span<'_>>) -> String {
        let mut result = String::new();

        for (span, kind) in &e.errors {
            if self.input.is_empty() {
                match kind {
                    VerboseErrorKind::Char(c) => {
                        write!(&mut result, "expected '{c}', got empty input\n\n").unwrap();
                    }
                    VerboseErrorKind::Context(s) => {
                        write!(&mut result, "in {s}, got empty input\n\n").unwrap();
                    }
                    VerboseErrorKind::Nom(e) => {
                        write!(&mut result, "in {e:?}, got empty input\n\n").unwrap();
                    }
                }
            } else {
                let line_num = span.line();
                let column_num = span.col();

                let line_begin = self.input[..span.byte_offset()]
                    .rfind('\n')
                    .map(|pos| pos + 1)
                    .unwrap_or(0);
                let line = self.input[line_begin..]
                    .lines()
                    .next()
                    .unwrap_or(&self.input[line_begin..])
                    .trim_end();

                let caret = '^';

                match kind {
                    VerboseErrorKind::Char(ch) => {
                        write!(
                            &mut result,
                            "at line {line_num}:\n\
                             {line}\n\
                             {caret:>column_num$}\n\
                             expected '{ch}'\n\n"
                        )
                        .unwrap();
                    }
                    VerboseErrorKind::Context(ctx) => {
                        write!(
                            &mut result,
                            "at line {line_num}, in {ctx}:\n\
                             {line}\n\
                             {caret:>column_num$}\n\n"
                        )
                        .unwrap();
                    }
                    VerboseErrorKind::Nom(e) => {
                        write!(
                            &mut result,
                            "at line {line_num}, in {e:?}:\n\
                             {line}\n\
                             {caret:>column_num$}\n\n"
                        )
                        .unwrap();
                    }
                };
            }
        }

        result
    }

    fn curr_addr(&self) -> u32 {
        (self.code.len() as u32) << 2
    }
}
