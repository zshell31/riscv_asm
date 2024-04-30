use std::fmt::{self, Debug, Display};

use nom::{character::complete::alphanumeric1, combinator::map_opt, error::context};
use phf::phf_map;

use crate::span::{IResult, Span};

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Reg(pub u32);

impl Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(reg_name(self.0))
    }
}

impl Debug for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl Reg {
    pub fn idx(&self) -> u32 {
        self.0
    }

    pub fn parse(input: Span<'_>) -> IResult<Self> {
        context(
            "Reg",
            map_opt(alphanumeric1, |s: Span<'_>| {
                REGS.get(*s).copied().map(Into::into)
            }),
        )(input)
    }
}

impl From<u32> for Reg {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

macro_rules! regs {
    ($($name:literal => $idx:literal),+) => {
        pub static REGS: phf::Map<&'static str, u32> = phf_map! {
            $(
                $name => $idx,
            )+
        };

        fn reg_name(idx: u32) -> &'static str {
            #[allow(unreachable_patterns)]
            match idx {
                $(
                    $idx => $name,
                )+
                _ => unreachable!()
            }
        }
    };
}

regs! {
    "zero"  => 0,   // x0 - Zero constant
    "ra"    => 1,   // x1 - Return address
    "sp"    => 2,   // x2 - Stack pointer
    "gp"    => 3,   // x3 - Global pointer
    "tp"    => 4,   // x4 - Thread pointer
    "t0"    => 5,   // x5 - Temporary
    "t1"    => 6,   // x6 - Temporary
    "t2"    => 7,   // x7 - Temporary
    "s0"    => 8,   // x8 - Saved pointer
    "fp"    => 8,   // x8 - Frame pointer
    "s1"    => 9,   // x9 - Saved register
    "a0"    => 10,  // x10 - Fn arg/return value
    "a1"    => 11,  // x11 - Fn arg/return value
    "a2"    => 12,  // x12 - Fn arg
    "a3"    => 13,  // x13 - Fn arg
    "a4"    => 14,  // x14 - Fn arg
    "a5"    => 15,  // x15 - Fn arg
    "a6"    => 16,  // x16 - Fn arg
    "a7"    => 17,  // x17 - Fn arg
    "s2"    => 18,  // x18 - Saved register
    "s3"    => 19,  // x19 - Saved register
    "s4"    => 20,  // x20 - Saved register
    "s5"    => 21,  // x21 - Saved register
    "s6"    => 22,  // x22 - Saved register
    "s7"    => 23,  // x23 - Saved register
    "s8"    => 24,  // x24 - Saved register
    "s9"    => 25,  // x25 - Saved register
    "s10"   => 26,  // x26 - Saved register
    "s11"   => 27,  // x27 - Saved register
    "t3"    => 28,  // x28 - Temporary
    "t4"    => 29,  // x29 - Temporary
    "t5"    => 30,  // x30 - Temporary
    "t6"    => 31   // x31 = Temporary
}
