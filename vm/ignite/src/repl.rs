use anyhow::Result;
use bytecode::Value;
use compiler::compile_string;
use rustyline::{DefaultEditor, error::ReadlineError};

use crate::{runtime::run, Runtime};

fn print_value(val:&Value) {
    match val {
        Value::Unit => println!("()"),
        Value::Bool(b) => println!("{}", b),
        Value::Int(i) => println!("{}", i),
        Value::Float(f) => println!("{}", f),
        Value::String(s) => println!("{}", s)
    }
}

pub fn ignite_repl() -> Result<()>  {
    let mut rl = DefaultEditor::new().unwrap();
    println!("Welcome to the RustScript REPL! Type /exit to exit.");
    println!();

    // let mut rt:Runtime = Runtime::default();

    loop {
        let readline = rl.readline(">>> ");

        match readline {
            Ok(inp) => {
                let inp = inp.trim().to_string();

                if inp.len() == 0 {
                    continue;
                }

                if inp.eq("/exit") {
                    println!("See you again!");
                    break;
                }

                rl.add_history_entry(inp.clone().trim()).unwrap();

                let compiled = compile_string(&inp);
                match compiled {
                    Ok(_) => (),
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                }

                let compiled = compiled.unwrap();

                // For now, make a new Runtime for each line
                // Later: try to introduce global state
                let mut rt = Runtime::new(compiled);
                rt = run(rt)?;

                let top = rt.operand_stack.last();

                match top {
                    Some(val) => print_value(val),
                    None => ()
                }

            },
            _ => ()
        }
    }

    Ok(())
}