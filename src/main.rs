use riscv_asm::program::Program;

fn main() {
    let program = r#"
mv  a0, zero
add s2, s3, s4
sub t0, t1, t2
    "#;

    match Program::parse(program) {
        Ok(program) => println!("{program}"),
        Err(e) => eprintln!("error: {e}"),
    };
}
