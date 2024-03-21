use anyhow::Result;
use compiler::compile_string;
use rustyline::{DefaultEditor, error::ReadlineError};

pub fn ignite_repl() -> Result<()>  {
    let mut rl = DefaultEditor::new().unwrap();
    println!("Welcome to the RustScript REPL! Type /exit to exit.");
    println!();

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

                let compiled = compile_string(&inp)?;
                
            },
            _ => ()
        }
    }

    Ok(())
}