use std::path::Path;
use std::time::Duration;

use anyhow::{Error, Result};
use bytecode::{builtin, read_bytecode};
use clap::Parser;
use compiler::compiler::compile_from_string;
use repl::ignite_repl;
use runtime::*;

pub use crate::error::*;
pub use crate::thread::*;

mod error;
mod micro_code;
mod repl;
mod runtime;
mod thread;

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

    /// Set custom time quantum for the VM.
    /// Default is 1000.
    #[arg(short, long)]
    quantum: Option<usize>,

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

    if let Some(quantum) = args.quantum {
        rt.set_time_quantum(Duration::from_millis(quantum as u64));
    }

    if args.debug > 0 {
        rt.set_debug_mode();
    }

    let rt = run(rt)?;

    // Print last value on op stack if there (result of program)
    let top = rt.current_thread.operand_stack.last();

    if let Some(val) = top {
        builtin::println_impl(val);
    }

    Ok(())
}

pub fn run_from_string(inp: &str) -> Result<Runtime> {
    let comp = compile_from_string(inp, true)?;
    let rt = Runtime::new(comp);
    let rt = run(rt)?;
    Ok(rt)
}

// E2E tests. Not sure how to use Runtime from tests/
#[cfg(test)]
mod tests {
    use crate::run_from_string;

    fn test_pass(inp: &str, exp: &str) {
        let rt = run_from_string(inp).expect("Should succeed");
        let res = rt.current_thread.operand_stack.last().clone();
        // if stack empty at the end, empty string
        let res_string = match res {
            Some(v) => v.to_string(),
            None => "".to_string(),
        };

        assert_eq!(res_string, exp);
        // Invariant: op stack must have length == 1 when value produced. accumulating operands on op stack can cause bugs
        if res.is_some() {
            assert_eq!(rt.current_thread.operand_stack.len(), 1);
        }
    }
    #[test]
    fn test_e2e_simple() {
        // int
        test_pass("2;", "");
        test_pass("2", "2");
        test_pass("2; 3; 4", "4");

        // float
        test_pass("2.23; 2; 4.56", "4.56");
        test_pass("2.23; 2; 4.56;", "");

        // bool
        test_pass("true; false", "false");
        test_pass("true; false;", "");

        // num ops
        test_pass("2+2*3", "8");
        test_pass("(2+2)*3", "12");
        test_pass("2*3+2", "8");
        test_pass("2-3+4/5*6-8+9", "0"); // because 4/5 = 0 not float
        test_pass("(2*3+(4-(6*5)))*(10-(20)*(3+2))", "1800");
        test_pass(
            "5.67 * 8.91 / 2.34 + 6.78 - 9.87 - 4.32",
            "14.179615384615389",
        );

        // bool ops
        test_pass("!true && false", "false");
        test_pass("false == (3 > 5)", "true");
        test_pass("false == (3 < 5)", "false");
        test_pass("(true || false) && false", "false");
        test_pass("true || false && false", "true"); // true || (false && false) - && has higher prec
    }

    #[test]
    fn test_e2e_blks() {
        // different combinations of semicolon/stmts: popped correctly
        test_pass("20 + { { 40; } 30 }", "50");
        test_pass("20 + { { 40 } 30 }", "50");
        test_pass("20 + { { 40 }; 30 }", "50");
        test_pass("20 + { { 40; } 30 }", "50");
        test_pass("20 + { { 40; }; 30 }", "50");
        test_pass("let y = 20 + { { 40 } 30 }; y", "50");
    }

    #[test]
    fn test_e2e_blks_pop() {
        // more tests to ensure popped correctly
        test_pass(
            r"
        let x = 2;
        {
           let y = 3;
           {
               let z = x;
               x = 10;
               z+y;
           }
        } 
        
        x
        ",
            "10",
        );

        test_pass("let x = 2; { {2+2;} }", "()");
        test_pass("let x = 2; { {2+2;} } {3}", "3");
        test_pass(
            r"
        let x = 2;
        {
           {
              2+2;
           }
        }
        
        x
        ",
            "2",
        );

        test_pass(
            "
        let x = 2; { 

            {
                {
                    2+2;
                }
            } 
        
        }
        ",
            "()",
        );
    }

    #[test]
    fn test_e2e_if_else() {
        test_pass(
            r"
        let x = { 
            let x = 10; 
            
            if !true {
                x+2 
               } else { 
                   x+4 
               }
               
        }; x
        ",
            "14",
        );

        test_pass("if false { 20 }; 30", "30");
        test_pass("if false { 20 }", "");
        test_pass("if false { 20; }", "");
        test_pass("if false { 20; };", "");

        // mix
        test_pass(
            r"
        let condition1 = true;
        let condition2 = false;
        
        let result = if condition1 && condition2 {
            2
        } else {
            if condition1 || condition2 {
                let x = 3;
                condition1 = false;
                x
            } else {
                let y = 4;
                y
            }
        };
        
        
        if !condition1 {
            result = result + 5;
        }
        
        result
        ",
            "8",
        );
    }

    #[test]
    fn test_e2e_lexical_scope() {
        let t = r"
        let x = 10;

        let result : int = {
            let x = 5;
            x
        };
        
        result
        ";
        test_pass(t, "5");

        let t = r"
        let x = 10;

        let result : int = {
            let x = 5;
            x
        };
        
        result + x
        ";
        test_pass(t, "15");

        // assign + new var
        let t = r"
        let x = 2; 
        let y = 0; 
        { 
            let x = 3; 
            y = 4 + x; 
        } 
        x+y
        ";
        test_pass(t, "9");

        // nested, and assign to outer
        let t = r"
        let x = 2;


        let z : int = {
           let y = 3;
           {
               let z = x;
               x = 10;
               z+y
           }
        };
        
        x+z
        ";
        test_pass(t, "15");
    }

    #[test]
    fn test_e2e_short_circuiting() {
        // test &&, || shortcircuit

        // &&
        test_pass(
            r"
        let x = 0;
        {x = 1; false} && {x=2; true}
        x",
            "1",
        );

        test_pass(
            r"
        let x = 0;
        {x = 1; true} && {x=2; true}
        x",
            "2",
        );

        test_pass(
            r"
        let x = 0;
        {x = 1; true} && {x=2; false}
        x",
            "2",
        );

        // stops at 2nd
        test_pass(
            r"
        let x = 0;
        {x=1; true} && {x=2; false} && {x=3; true}
        x",
            "2",
        );

        // goes till last
        test_pass(
            r"
        let x = 0;
        {x=1; true} && {x=2; true} && {x=3; false}
        x",
            "3",
        );

        // ||
        test_pass(
            r"
        let x = 0;
        {x = 1; true} || {x=2; true}
        x",
            "1",
        );

        test_pass(
            r"
        let x = 0;
        {x = 1; false} || {x=2; true}
        x",
            "2",
        );

        test_pass(
            r"
        let x = 0;
        {x = 1; false} || {x=2; false}
        x",
            "2",
        );

        // stops at 2nd
        test_pass(
            r"
        let x = 0;
        {x=1; false} || {x=2; true} || {x=3; true}
        x",
            "2",
        );

        // 3rd
        test_pass(
            r"
        let x = 0;
        {x=1; false} || {x=2; false} || {x=3; false}
        x",
            "3",
        );
    }

    #[test]
    fn test_e2e_loops() {
        let t = r"
        let x = 0;
        loop x < 3 {
            x = x + 1;
        }
        x
        ";
        test_pass(t, "3");

        // loop-01.rst
        let t = r"
        let i = 0;

        loop {
          if i > 10 {
            break;
          }
          
          i = i + 1;
        }
        
        i
        
        ";
        test_pass(t, "11");

        let t = r"
        let x = 0;
        loop x < 3 {
            x = x + 1;
            
            if x == 2 {
                break;
              }
        }
        x
        ";
        test_pass(t, "2");

        // sum of naturals - nested loop
        let t = r"
        let count = 0;
        let x = 0;
        let end = 10;
        
        loop x < end|| x == end{
            let j = 0;
            
            loop j < x {
                count = count + 1;
                j = j + 1;
            }
            
            x = x + 1;
        }
        
        count
        ";
        test_pass(t, "55");

        // nested, both have break - break targets correct loop each time
        let t = r"
        let count = 0;
        let x = 0;
        
        loop x < 10 || x == 10 {
            let j = 0;
            
            
            loop {
                    if !(j < x) {
                          break;
                    }
                    
                count = count + 1;
                j = j + 1;
                
                if j == 5 {
                    break;
                }
            }
            
            x = x + 1;
            
            if x == 7 {
                break;
            }
        }
        
        count
        ";
        test_pass(t, "20");

        // Triple nested
        test_pass(
            r"
        let count = 0;

        let x = 0;
        loop x < 5 || x == 5 {
            let y = 0;
            
            loop y < 5 || y == 5 {
                let z = 0;
                
                loop {
                        if !(z < 5 || z == 5) {
                            break;
                        }
                        
                    count = count + 1;
                    z = z + 1;
        
                    if z == 3 {
                        break;
                    }
                }
                
                y = y + 1;
        
                if y == 3 {
                    break;
                }
            }
            
            x = x + 1;
        
            if x == 3 {
                break;
            }
        }
        
        count
        ",
            "27",
        );
    }

    #[test]
    fn test_e2e_fib() {
        // loop-fib-01.rst
        let t = r"
        let n : int = 10; // Calculate the 10th (0 idx) Fibonacci number = 55
        let fib_prev : int = 0;
        let fib_current : int = 1;
        let fib_next : int = 0; 
        
        let i = 1; // Start from the 1st Fibonacci number
        
        loop i < n {
            fib_next = fib_prev + fib_current;
            fib_prev = fib_current;
            fib_current = fib_next;
            i = i + 1;
        }
        
        if n == 0 {
            fib_prev
        } else {
            fib_current
        }
        ";
        test_pass(t, "55");
    }
}
