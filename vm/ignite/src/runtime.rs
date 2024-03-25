use std::{cell::RefCell, rc::Rc};

use anyhow::Result;
use bytecode::{ByteCode, Environment, StackFrame, Symbol, Value};

use crate::{micro_code, VmError};

/// The runtime for each thread of execution.
#[derive(Debug, Default)]
pub struct Runtime {
    pub env: Rc<RefCell<Environment>>,
    pub operand_stack: Vec<Value>,
    pub runtime_stack: Vec<StackFrame>,
    pub instrs: Vec<ByteCode>,
    pub pc: usize,
}

impl Runtime {
    pub fn new(instrs: Vec<ByteCode>) -> Self {
        Runtime {
            env: Environment::new_wrapped(),
            operand_stack: Vec::new(),
            runtime_stack: Vec::new(),
            instrs,
            ..Default::default()
        }
    }
}

/// Run the program until it is done.
///
/// # Arguments
///
/// * `rt` - The runtime to run.
///
/// # Returns
///
/// The runtime after the program has finished executing.
///
/// # Errors
///
/// If an error occurs during execution.
pub fn run(mut rt: Runtime) -> Result<Runtime> {
    loop {
        let instr = rt
            .instrs
            .get(rt.pc)
            .ok_or(VmError::PcOutOfBounds(rt.pc))?
            .clone();
        rt.pc += 1;

        let is_done = execute(&mut rt, instr)?;
        if is_done {
            break;
        }
    }

    Ok(rt)
}

/// Execute a single instruction, returning whether the program is done.
///
/// # Arguments
///
/// * `rt` - The runtime to execute the instruction on.
///
/// * `instr` - The instruction to execute.
///
/// # Returns
///
/// Whether the program is done executing.
///
/// # Errors
///
/// If an error occurs during execution.
pub fn execute(rt: &mut Runtime, instr: ByteCode) -> Result<bool> {
    match instr {
        ByteCode::DONE => return Ok(true),
        ByteCode::ASSIGN(sym) => micro_code::assign(rt, sym)?,
        ByteCode::LD(sym) => micro_code::ld(rt, sym)?,
        ByteCode::LDC(val) => micro_code::ldc(rt, val)?,
        ByteCode::POP => micro_code::pop(rt)?,
        ByteCode::UNOP(op) => micro_code::unop(rt, op)?,
        ByteCode::BINOP(op) => micro_code::binop(rt, op)?,
        ByteCode::JOF(pc) => micro_code::jof(rt, pc)?,
        ByteCode::GOTO(pc) => micro_code::goto(rt, pc)?,
        ByteCode::RESET(t) => micro_code::reset(rt, t)?,
        ByteCode::ENTERSCOPE(syms) => micro_code::enter_scope(rt, syms)?,
        ByteCode::EXITSCOPE => micro_code::exit_scope(rt)?,
    }
    Ok(false)
}

/// Extend the current environment with new symbols and values.
///
/// # Arguments
///
/// * `rt` - The runtime to extend the environment of.
///
/// * `syms` - The symbols to add to the environment.
///
/// * `vals` - The values to add to the environment.
///
/// # Errors
///
/// If the symbols and values are not the same length.
pub fn extend_environment(rt: &mut Runtime, syms: Vec<Symbol>, vals: Vec<Value>) -> Result<()> {
    if syms.len() != vals.len() {
        return Err(VmError::IllegalArgument(
            "symbols and values must be the same length".to_string(),
        )
        .into());
    }

    let current_env = Rc::clone(&rt.env);
    let new_env = Environment::new_wrapped();
    new_env.borrow_mut().set_parent(current_env);

    for (sym, val) in syms.into_iter().zip(vals.into_iter()) {
        new_env.borrow_mut().set(sym, val);
    }

    rt.env = new_env;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Ok;
    use bytecode::{BinOp, UnOp};

    #[test]
    fn test_pc() {
        let instrs = vec![
            ByteCode::ldc(42),
            ByteCode::POP,
            ByteCode::ldc(42),
            ByteCode::POP,
            ByteCode::DONE,
        ];
        let rt = Runtime::new(instrs);
        let rt = run(rt).unwrap();
        assert_eq!(rt.pc, 5);

        let rt = Runtime::new(vec![
            ByteCode::ldc(false),
            ByteCode::JOF(3),
            ByteCode::POP, // This will panic since there is no value on the stack
            ByteCode::DONE,
        ]);
        let rt = run(rt).unwrap();
        assert_eq!(rt.pc, 4);

        let rt = Runtime::new(vec![
            ByteCode::ldc(true),
            ByteCode::JOF(3), // jump to pop instruction
            ByteCode::DONE,
            ByteCode::POP, // This will panic since there is no value on the stack
            ByteCode::DONE,
        ]);
        let rt = run(rt).unwrap();
        assert_eq!(rt.pc, 3);

        let rt = Runtime::new(vec![
            ByteCode::GOTO(2),
            ByteCode::POP, // This will panic since there is no value on the stack
            ByteCode::DONE,
        ]);
        let rt = run(rt).unwrap();
        assert_eq!(rt.pc, 3);
    }

    #[test]
    fn test_arithmetic() {
        // 42 + 42
        let instrs = vec![
            ByteCode::ldc(42),
            ByteCode::ldc(42),
            ByteCode::BINOP(BinOp::Add),
            ByteCode::DONE,
        ];
        let rt = Runtime::new(instrs);
        let rt = run(rt).unwrap();
        assert_eq!(rt.operand_stack, vec![Value::Int(84)]);

        // -(42 - 123)
        let instrs = vec![
            ByteCode::ldc(42),
            ByteCode::ldc(123),
            ByteCode::BINOP(BinOp::Sub),
            ByteCode::UNOP(UnOp::Neg),
            ByteCode::DONE,
        ];
        let rt = Runtime::new(instrs);
        let rt = run(rt).unwrap();
        assert_eq!(rt.operand_stack, vec![Value::Int(81)]);

        // (2 * 3) > 9
        let instrs = vec![
            ByteCode::ldc(2),
            ByteCode::ldc(3),
            ByteCode::BINOP(BinOp::Mul),
            ByteCode::ldc(9),
            ByteCode::BINOP(BinOp::Gt),
            ByteCode::DONE,
        ];
        let rt = Runtime::new(instrs);
        let rt = run(rt).unwrap();
        assert_eq!(rt.operand_stack, vec![Value::Bool(false)]);
    }

    #[test]
    fn test_assignment() {
        let instrs = vec![
            ByteCode::ldc(42),
            ByteCode::ASSIGN("x".to_string()),
            ByteCode::ldc(43),
            ByteCode::ASSIGN("y".to_string()),
            ByteCode::ldc(44),
            ByteCode::ASSIGN("x".to_string()),
            ByteCode::DONE,
        ];
        let rt = Runtime::new(instrs);
        let rt = run(rt).unwrap();
        assert_eq!(rt.env.borrow().get(&"x".to_string()), Some(Value::Int(44)));
        assert_eq!(rt.env.borrow().get(&"y".to_string()), Some(Value::Int(43)));
    }

    #[test]
    fn test_extend_environment() -> Result<()> {
        let mut rt = Runtime::new(vec![]);
        rt.env.borrow_mut().set("a", 42);
        rt.env.borrow_mut().set("b", 123);

        assert!(
            extend_environment(&mut rt, vec!["c".to_string(), "d".to_string()], vec![]).is_err()
        );

        extend_environment(
            &mut rt,
            vec!["c".to_string(), "d".to_string()],
            vec![Value::Float(12.3), Value::Bool(true)],
        )?;

        assert_eq!(rt.env.borrow().get(&"a".to_string()), Some(Value::Int(42)));
        assert_eq!(rt.env.borrow().get(&"b".to_string()), Some(Value::Int(123)));
        assert_eq!(
            rt.env.borrow().get(&"c".to_string()),
            Some(Value::Float(12.3))
        );
        assert_eq!(
            rt.env.borrow().get(&"d".to_string()),
            Some(Value::Bool(true))
        );

        Ok(())
    }
}
