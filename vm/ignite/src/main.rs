pub use crate::error::*;
pub use crate::frame::Frame;
pub use crate::runtime::Runtime;

use anyhow::Result;
use bytecode::read_bytecode;
use clap::Parser;
use repl::ignite_repl;
use std::path::Path;

mod error;
mod frame;
mod micro_code;
mod runtime;
mod repl;

#[derive(Parser, Debug)]
#[command(name = "Ignite")]
#[command(version = "0.1.0")]
#[command(about = "Virtual Machine for RustScript", long_about = None)]
struct Args {
    /// File name of the program to run, must be a .o2 file.
    file: String,

    /// If true, launch in REPL mode. False by default.
    #[arg(long, short)]
    repl:bool,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,
}

fn main() -> Result<()> {
    let args = Args::parse();

    if args.repl {
        ignite_repl()?;
    }

    // Check if the file exists
    if !Path::new(&args.file).exists() {
        return Err(VmError::FileDoesNotExist(args.file).into());
    }

    // check file extension
    if Path::new(&args.file).extension().unwrap() != "o2" {
        return Err(VmError::NotO2File(args.file).into());
    }

    // Deserialize the program
    let mut file = std::fs::File::open(&args.file)?;
    let bytecode_vec = read_bytecode(&mut file)?;

    let rt = Runtime::new(bytecode_vec);
    runtime::run(rt)?;

    Ok(())
}
