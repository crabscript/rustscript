use std::path::Path;

use anyhow::{Error, Result};
use bytecode::read_bytecode;
use clap::Parser;
use repl::{ignite_repl, print_value};

pub use crate::error::*;
pub use crate::runtime::Runtime;

mod error;
mod micro_code;
mod repl;
mod runtime;

#[derive(Parser, Debug)]
#[command(name = "Ignite")]
#[command(version = "0.1.0")]
#[command(about = "Virtual Machine for RustScript", long_about = None)]
struct Args {
    /// File name of the program to run, must be a .o2 file.
    file: Option<String>,

    /// If true, launch in REPL mode. False by default.
    #[arg(long, short)]
    repl: bool,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    /// If present, does not type check in REPL. Ignored if only running bytecode.
    #[arg(short)]
    notype: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let file_provided = args.file.is_some();

    if args.repl {
        // TODO: if file provided, run the file and pass generated context to REPL
        ignite_repl(!args.notype)?;
        return Ok(()); // REPL done: exit
    } else if !args.repl && !file_provided {
        return Err(Error::msg("File should be provided if not launching REPL."));
    }

    let file = args.file.expect("File was provided");

    // Check if the file exists
    if !Path::new(&file).exists() {
        return Err(VmError::FileDoesNotExist(file).into());
    }

    // check file extension
    if Path::new(&file).extension().unwrap() != "o2" {
        return Err(VmError::NotO2File(file).into());
    }

    // Deserialize the program
    let mut file = std::fs::File::open(file)?;
    let bytecode_vec = read_bytecode(&mut file)?;

    let mut rt = Runtime::new(bytecode_vec);
    rt = runtime::run(rt)?;

    // Print last value on op stack if there (result of program)
    let top = rt.operand_stack.last();

    if let Some(val) = top {
        print_value(val)
    }

    Ok(())
}
