use std::fmt::{self, Display};

use nom::{
    branch::alt,
    bytes::complete::{take_until, take_while},
    character::complete::{multispace0, space0},
    combinator::rest,
    error::{Error, ErrorKind},
    Err, Finish, IResult,
};

use crate::instr::Instr;

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
