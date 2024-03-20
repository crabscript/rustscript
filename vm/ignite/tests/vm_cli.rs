use anyhow::Result;
use assert_cmd::prelude::*;
use bytecode::ByteCode;
use predicates::prelude::*;
use std::process::Command;

const IGNITE_BINARY: &str = "ignite";

#[test]
fn file_doesnt_exist() -> Result<()> {
    let mut cmd = Command::cargo_bin(IGNITE_BINARY)?;

    cmd.arg("test/file/doesnt/exist");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("File does not exist"));

    Ok(())
}

#[test]
fn file_not_o2() -> Result<()> {
    let mut cmd = Command::cargo_bin(IGNITE_BINARY)?;

    cmd.arg("Cargo.toml");
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("File is not a .o2 file"));

    Ok(())
}

#[test]
fn run_simple_program() -> Result<()> {
    let mut cmd = Command::cargo_bin(IGNITE_BINARY)?;

    let bytecode = vec![
        ByteCode::ldc(42),
        ByteCode::ldc(15),
        ByteCode::BINOP(bytecode::BinOp::Add),
        ByteCode::POP,
        ByteCode::DONE,
    ];

    let mut file = std::fs::File::create("./simple.o2")?;
    bytecode::write_bytecode(&bytecode, &mut file)?;

    cmd.arg("./simple.o2");
    cmd.assert().success();

    std::fs::remove_file("./simple.o2")?;

    Ok(())
}
