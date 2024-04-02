use anyhow::Result;
use bytecode::Value;
use compiler::compiler;
use rustyline::DefaultEditor;

use crate::{runtime::run, Runtime};

pub fn print_value(val: &Value) {
    match val {
        Value::Unitialized => println!("Uninitialized"),
        Value::Unit => println!("()"),
        Value::Bool(b) => println!("{}", b),
        Value::Int(i) => println!("{}", i),
        Value::Float(f) => println!("{}", f),
        Value::String(s) => println!("{}", s),
        Value::Closure {
            sym,
            prms,
            addr,
            env: _,
        } => println!("Closure - sym: {}, prms: {:?}, addr: {}", sym, prms, addr),
    }
}

pub fn ignite_repl(type_check: bool) -> Result<()> {
    let mut rl = DefaultEditor::new().unwrap();
    println!("Welcome to the RustScript REPL! Type /exit to exit.");
    println!();

    // let mut rt:Runtime = Runtime::default();

    loop {
        let readline = rl.readline(">>> ");

        if let Ok(inp) = readline {
            let inp = inp.trim().to_string();

            if inp.is_empty() {
                continue;
            }

            if inp.eq("/exit") {
                println!("See you again!");
                break;
            }

            rl.add_history_entry(inp.clone().trim()).unwrap();

            let compiled = compiler::compile_from_string(&inp, type_check);
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
            let run_res = run(rt);

            match run_res {
                Ok(_) => (),
                Err(err) => {
                    println!("[RuntimeError]: {}", err);
                    continue;
                }
            }

            rt = run_res.unwrap();

            let top = rt.operand_stack.last();

            if let Some(val) = top {
                print_value(val)
            }
        }
    }

    Ok(())
}
