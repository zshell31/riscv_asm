use anyhow::anyhow;
use riscv_asm::{error::AsmError, program::Program};

fn dump_code(input: &str) -> Result<(), AsmError<'_>> {
    Program::parse(input)?.dump_code()
}

fn main() -> anyhow::Result<()> {
    let program = r#"
start:      mv a0, zero
counter:    addi a0, a0, 1
            beq zero, zero, counter
    "#;

    dump_code(program).map_err(|e| anyhow!("{}", e.to_str(program)))
}
