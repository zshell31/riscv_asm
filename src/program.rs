use anyhow::anyhow;
use std::collections::HashMap;

use nom::{
    branch::alt,
    bytes::complete::{take_until, take_while},
    character::complete::{anychar, char, multispace0, space0, space1},
    combinator::{eof, map, opt, rest},
    error::Error,
    sequence::{pair, preceded, terminated},
    Finish, IResult,
};

use crate::{imm::parse_sym, instr::Instr};

#[derive(Debug, Default)]
pub struct Program {
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

impl Program {
    pub fn resolve(&self, sym: &str) -> anyhow::Result<i32> {
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

// impl Display for Program {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         for (addr, code) in self.code.iter().enumerate() {
//             writeln!(
//                 f,
//                 "{:08x}:  {:02x} {:02x} {:02x} {:02x}",
//                 addr << 2,
//                 (code >> 24) & 0xff,
//                 (code >> 16) & 0xff,
//                 (code >> 8) & 0xff,
//                 code & 0xff
//             )?;
//         }

//         Ok(())
//     }
// }

enum Line {
    Instr(Instr),
    Label(String, Option<Instr>),
}

impl Line {
    fn parse(input: &str) -> IResult<&str, Self> {
        map(
            pair(
                alt((Self::parse_instr, Self::parse_label)),
                opt(preceded(space1, Self::parse_comment)),
            ),
            |(this, _)| this,
        )(input)
    }

    fn parse_instr(input: &str) -> IResult<&str, Self> {
        map(Instr::parse, Self::Instr)(input)
    }

    fn parse_label(input: &str) -> IResult<&str, Self> {
        map(
            pair(
                terminated(parse_sym, char(':')),
                opt(preceded(space1, Instr::parse)),
            ),
            |(label, instr)| Self::Label(label.to_string(), instr),
        )(input)
    }

    fn parse_comment(input: &str) -> IResult<&str, ()> {
        map(preceded(char('#'), anychar), |_| ())(input)
    }
}

impl Program {
    pub fn parse(input: &str) -> Result<Self, Error<&str>> {
        let mut program = Self::default();
        program.parse_code(input).finish().map(|_| program)
    }

    fn parse_code<'i>(&mut self, input: &'i str) -> IResult<&'i str, ()> {
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
