use nom::error::{ErrorKind, FromExternalError, ParseError};
use nom::IResult as NomResult;

use crate::span::Span;

pub type IResult<'i, O> = NomResult<Span<'i>, O, AsmError<'i>>;

#[derive(Debug)]
pub struct AsmError<'i> {
    pub span: Span<'i>,
    pub kind: AsmErrorKind,
}

impl<'i> AsmError<'i> {
    pub fn with_kind(self, kind: AsmErrorKind) -> Self {
        Self {
            span: self.span,
            kind,
        }
    }

    pub fn to_str(&self, input: &str) -> String {
        let line_num = self.span.line();
        let column_num = self.span.col();

        let line_begin = input[..self.span.byte_offset()]
            .rfind('\n')
            .map(|pos| pos + 1)
            .unwrap_or(0);
        let line = input[line_begin..]
            .lines()
            .next()
            .unwrap_or(&input[line_begin..])
            .trim_end();

        let caret = '^';
        let kind = &self.kind;

        format!(
            "{kind} at line {line_num}:\n\
             {line}\n\
             {caret:>column_num$}\n"
        )
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AsmErrorKind {
    #[error("Nom '{}'", ErrorKind::description(.0))]
    Nom(ErrorKind),
    #[error("Invalid Instr")]
    InvalidInstr,
    #[error("Invalid OpCode")]
    InvalidOpCode,
    #[error("Invalid Reg")]
    InvalidReg,
    #[error("Invalid Imm")]
    InvalidImm,
    #[error("Invalid Pseudo instr")]
    InvalidPseudo,
    #[error("Unknown Sym")]
    UnknownSym,
}

impl<'i> ParseError<Span<'i>> for AsmError<'i> {
    fn from_error_kind(span: Span<'i>, kind: ErrorKind) -> Self {
        Self {
            span,
            kind: AsmErrorKind::Nom(kind),
        }
    }

    fn append(_: Span<'i>, _: ErrorKind, other: Self) -> Self {
        other
    }
}

impl<'i, E> FromExternalError<Span<'i>, E> for AsmError<'i> {
    fn from_external_error(input: Span<'i>, kind: ErrorKind, _: E) -> Self {
        Self::from_error_kind(input, kind)
    }
}
