pub use literify::literify;
use nom::{character::complete::alpha0, combinator::map_opt, IResult};
pub use phf::phf_map;

use crate::op_code::OpCode;

impl Pseudo {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        map_opt(alpha0, |s| PSEUDO.get(s).copied())(input)
    }
}

macro_rules! pseudo {
    ($($name:ident => $op_code:ident),+) => {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, Copy)]
        pub enum Pseudo {
            $(
                $name,
            )+
        }

        impl Pseudo {
            pub fn op_code(&self) -> OpCode {
                match self {
                    $(
                        Self::$name => OpCode::$op_code,
                    )+
                }
            }
        }

        literify! {
            pub static PSEUDO: phf::Map<&'static str, Pseudo> = phf_map! {
                $(
                    ~($name) => Pseudo::$name,
                )+
            };
        }
    };
}

pseudo! {
    mv => addi
}
