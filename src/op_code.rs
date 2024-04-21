use literify::literify;
use phf::phf_map;

macro_rules! mask {
    (R, $opcode:literal, $f3:literal, $f7:literal) => {
        (($f7 & 0x7f) << 25) | (($f3 & 0x7) << 12) | ($opcode & 0x7f)
    };
}

#[derive(Debug, Clone, Copy)]
pub enum OpCodeTy {
    R,
}

macro_rules! op_code {
    ($($name:ident : $ty:ident, $($t:tt),+);+ $(;)?) => {
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, Copy)]
        #[repr(u32)]
        pub enum OpCode {
            $(
                $name = mask!($ty, $($t),*),
            )+
        }

        impl OpCode {
            pub fn ty(&self) -> OpCodeTy {
                match self {
                    $(
                        Self::$name => OpCodeTy::$ty,
                    )+
                }
            }
        }

        literify! {
            pub static INSTR: phf::Map<&'static str, OpCode> = phf_map! {
                $(
                    ~($name) => OpCode::$name,
                )+
            };
        }

    };
}

op_code! {
    add     : R, 0x33, 0x0, 0x00;
    sub     : R, 0x33, 0x0, 0x20;
    xor     : R, 0x33, 0x4, 0x00;
    or      : R, 0x33, 0x6, 0x00;
    and     : R, 0x33, 0x7, 0x00;
    sll     : R, 0x33, 0x1, 0x00;
    srl     : R, 0x33, 0x5, 0x00;
    sra     : R, 0x33, 0x5, 0x20;
    slt     : R, 0x33, 0x2, 0x00;
    sltu    : R, 0x33, 0x3, 0x00;
}
