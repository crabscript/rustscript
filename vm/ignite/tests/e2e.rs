use anyhow::Result;
use assert_cmd::prelude::*;
use compiler::compiler::compile_from_string;
use predicates::prelude::*;
use std::process::Command;

const IGNITE_BINARY: &str = "ignite";
const OXIDATE_BINARY: &str = "oxidate";

// Have to use random file name because tests run in parallel
// With fixed filename we get errors due to race conditions
fn test_pass(inp: &str, exp: &str) -> Result<()> {
    let file_num = rand::random::<u128>().to_string();
    let file_name = format!("./{file_num}.o2");

    let mut cmd = Command::cargo_bin(IGNITE_BINARY)?;
    let comp = compile_from_string(inp, true)?;

    let mut file = std::fs::File::create(file_name.clone())?;
    bytecode::write_bytecode(&comp, &mut file)?;

    cmd.arg(file_name.clone());
    let exp = if exp.is_empty() {
        String::from("")
    } else {
        format!("{}\n", exp)
    };
    cmd.assert().success().stdout(predicate::eq(exp));

    std::fs::remove_file(file_name)?;

    Ok(())
}

// Test files in example/
// file_name is expected to be prefix before .rst
fn test_file(file_name: &str, exp: &str) -> Result<()> {
    let file_name_rst = format!("../../example/{file_name}.rst");

    let mut cmd = Command::cargo_bin(OXIDATE_BINARY)?;
    cmd.arg(file_name_rst.clone()).assert().success();

    dbg!(format!("{file_name}.o2"));

    let file_name_o2 = format!("{file_name}.o2");

    let mut cmd_vm = Command::cargo_bin(IGNITE_BINARY)?;

    let exp = if exp.is_empty() {
        String::from("")
    } else {
        format!("{}\n", exp)
    };

    cmd_vm
        .arg(file_name_o2.clone())
        .assert()
        .success()
        .stdout(predicate::eq(exp));
    std::fs::remove_file(format!("./{file_name}.o2"))?;

    Ok(())
}

#[test]
fn test_e2e_example_folder() -> Result<()> {
    test_file("block-01", "true")?;
    test_file("loop-01", "11")?;
    test_file("loop-02", "55")?;
    test_file("loop-03", "55")?;
    test_file("loop-04", "20")?;
    test_file("type-01", "33")?;
    Ok(())
}

#[test]
fn test_e2e_simple() -> Result<()> {
    // int
    test_pass("2;", "")?;
    test_pass("2", "2")?;
    test_pass("2; 3; 4", "4")?;

    // float
    test_pass("2.23; 2; 4.56", "4.56")?;
    test_pass("2.23; 2; 4.56;", "")?;

    // bool
    test_pass("true; false", "false")?;
    test_pass("true; false;", "")?;

    // num ops
    test_pass("2+2*3", "8")?;
    test_pass("(2+2)*3", "12")?;
    test_pass("2*3+2", "8")?;
    test_pass("2-3+4/5*6-8+9", "0")?; // because 4/5 = 0 not float
    test_pass("(2*3+(4-(6*5)))*(10-(20)*(3+2))", "1800")?;
    test_pass(
        "5.67 * 8.91 / 2.34 + 6.78 - 9.87 - 4.32",
        "14.179615384615389",
    )?;

    // bool ops
    test_pass("!true && false", "false")?;
    test_pass("false == (3 > 5)", "true")?;
    test_pass("false == (3 < 5)", "false")?;
    test_pass("(true || false) && false", "false")?;
    test_pass("true || false && false", "true")?; // true || (false && false) - && has higher prec

    Ok(())
}

#[test]
fn test_e2e_blks() -> Result<()> {
    // different combinations of semicolon/stmts: popped correctly
    test_pass("20 + { { 40; } 30 }", "50")?;
    test_pass("20 + { { 40 } 30 }", "50")?;
    test_pass("20 + { { 40 }; 30 }", "50")?;
    test_pass("20 + { { 40; } 30 }", "50")?;
    test_pass("20 + { { 40; }; 30 }", "50")?;
    test_pass("let y = 20 + { { 40 } 30 }; y", "50")?;

    Ok(())
}

#[test]
fn test_e2e_blks_pop() -> Result<()> {
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
    )?;

    test_pass("let x = 2; { {2+2;} }", "()")?;
    test_pass("let x = 2; { {2+2;} } {3}", "3")?;
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
    )?;

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
    )?;

    Ok(())
}

#[test]
fn test_e2e_if_else() -> Result<()> {
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
    )?;

    test_pass("if false { 20 }; 30", "30")?;
    test_pass("if false { 20 }", "")?;
    test_pass("if false { 20; }", "")?;
    test_pass("if false { 20; };", "")?;

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
    )?;

    Ok(())
}

#[test]
fn test_e2e_lexical_scope() -> Result<()> {
    let t = r"
    let x = 10;

    let result : int = {
        let x = 5;
        x
    };
    
    result
    ";
    test_pass(t, "5")?;

    let t = r"
    let x = 10;

    let result : int = {
        let x = 5;
        x
    };
    
    result + x
    ";
    test_pass(t, "15")?;

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
    test_pass(t, "9")?;

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
    test_pass(t, "15")?;

    Ok(())
}

#[test]
fn test_e2e_short_circuiting() -> Result<()> {
    // test &&, || shortcircuit

    // &&
    test_pass(
        r"
    let x = 0;
    {x = 1; false} && {x=2; true}
    x",
        "1",
    )?;

    test_pass(
        r"
    let x = 0;
    {x = 1; true} && {x=2; true}
    x",
        "2",
    )?;

    test_pass(
        r"
    let x = 0;
    {x = 1; true} && {x=2; false}
    x",
        "2",
    )?;

    // stops at 2nd
    test_pass(
        r"
    let x = 0;
    {x=1; true} && {x=2; false} && {x=3; true}
    x",
        "2",
    )?;

    // goes till last
    test_pass(
        r"
    let x = 0;
    {x=1; true} && {x=2; true} && {x=3; false}
    x",
        "3",
    )?;

    // ||
    test_pass(
        r"
    let x = 0;
    {x = 1; true} || {x=2; true}
    x",
        "1",
    )?;

    test_pass(
        r"
    let x = 0;
    {x = 1; false} || {x=2; true}
    x",
        "2",
    )?;

    test_pass(
        r"
    let x = 0;
    {x = 1; false} || {x=2; false}
    x",
        "2",
    )?;

    // stops at 2nd
    test_pass(
        r"
    let x = 0;
    {x=1; false} || {x=2; true} || {x=3; true}
    x",
        "2",
    )?;

    // 3rd
    test_pass(
        r"
    let x = 0;
    {x=1; false} || {x=2; false} || {x=3; false}
    x",
        "3",
    )?;

    Ok(())
}

#[test]
fn test_e2e_loops() -> Result<()> {
    let t = r"
    let x = 0;
    loop x < 3 {
        x = x + 1;
    }
    x
    ";
    test_pass(t, "3")?;

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
    test_pass(t, "11")?;

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
    test_pass(t, "2")?;

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
    test_pass(t, "55")?;

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
    test_pass(t, "20")?;

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
    )?;

    Ok(())
}

#[test]
fn test_e2e_fib() -> Result<()> {
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
    test_pass(t, "55")?;

    Ok(())
}

#[test]
fn test_e2e_fn_decl() -> Result<()> {
    let t = r"
    fn fac(n: int) -> int {
        2 + n
    }
    
    fac(2)
    ";
    test_pass(t, "4")?;

    let hof = r"
    fn fac_tail(n: int) -> int {
        fn tail(n: int, acc: int) -> int {
            if n == 0 {
                return acc;
            } else {
                return tail(n-1, acc * n);
            }   
        }
    
        tail(n, 1)
    }
    
    let res : int = fac_tail(4);
    res // 24
    ";
    test_pass(hof, "24")?;

    let hof = r"
    fn adder(x: int) -> fn(int) -> int {
        fn g(y: int) -> int {
            x + y
        }

        return g;
    }

    let add5 : fn(int) -> int = adder(5);
    add5(10)
    ";
    test_pass(hof, "15")?;

    // apply fn passed in
    let hof = r"
    fn apply(f: fn(int) -> int, x: int) -> int {
        f(x)
    }

    fn add(x: int) -> int {
        return x + 5;
    }

    apply(add, 9)
    ";
    test_pass(hof, "14")?;

    Ok(())
}
