pub mod compiler;

use anyhow::Result;
use clap::Parser;
use std::{io::Read, path::Path};

use crate::compiler::{compile_from_string, CompileError};

const RST: &str = "rst";

#[derive(clap::Parser, Debug)]
#[command(name = "Oxidate")]
#[command(version = "0.1.0")]
#[command(about = "Compiler for RustScript", long_about = None)]
struct Args {
    /// File containing RustScript code. Must have extension .rst
    file: String,

    /// Output name (to be suffixed by .o2)
    #[arg(short, long)]
    out: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let file = args.file;

    if !Path::new(&file).exists() {
        let err = format!("File '{}' does not exist", file);
        return Err(CompileError::new(&err).into());
    }

    match Path::new(&file).extension() {
        Some(ext) => {
            if ext != RST {
                let err = format!("File {} does not have extension .{RST}", file);
                return Err(CompileError::new(&err).into());
            }
        }
        None => {
            let err = format!("File {} does not have extension .{RST}", file);
            return Err(CompileError::new(&err).into());
        }
    }

    let mut code: String = String::new();
    std::fs::File::open(&file)
        .expect("File should exist")
        .read_to_string(&mut code)?;

    let bytecode = compile_from_string(&code)?;
    println!("{:?}", bytecode);

    Ok(())
}
