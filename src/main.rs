use riscv_asm::program::Program;

fn main() -> anyhow::Result<()> {
    let program = Program::parse(
        r#"
start:      mv a0, zero
counter:    addis a0, a0, 1
    "#,
    )?;
    //     let program = Program::parse(
    //         r#"
    // start:      mv a0, zero
    // counter:    addi a0, a0, 1
    //             beq zero, zero, counter
    //     "#,
    //     )?;

    program.dump_code()
}
