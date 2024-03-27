pub mod compiler;

use anyhow::Result;
use bytecode::write_bytecode;
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
    let path = Path::new(&file);

    if !path.exists() {
        let err = format!("File '{}' does not exist", file);
        return Err(CompileError::new(&err).into());
    }

    match path.extension() {
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

    let out_name;
    if let Some(name) = args.out {
        out_name = name;
    } else {
        out_name = path
            .file_stem()
            .expect("File exists")
            .to_owned()
            .into_string()
            .expect("File name should be valid string");
    }

    // Write to .o2 file
    let bc_name = format!("{}.o2", out_name);
    let mut bc_file = std::fs::File::create(&bc_name).unwrap();
    write_bytecode(&bytecode, &mut bc_file)?;

    println!("Compiled successfully to {}", bc_name);

    Ok(())
}
