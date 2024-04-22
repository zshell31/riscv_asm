use nom::{character::complete::digit1, combinator::map_res, IResult};

#[derive(Debug, Clone, Copy)]
pub struct Imm(pub u32);

impl From<u32> for Imm {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

impl Imm {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        map_res(digit1, |s: &str| s.parse::<u32>().map(Into::into))(input)
    }

    pub fn code(&self, shift: u32) -> u32 {
        (self.0 & 0xfff) << shift
    }
}
