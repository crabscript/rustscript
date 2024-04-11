use anyhow::Result;
use bytecode::ByteCode;

use crate::{micro_code, Thread, VmError};

/// The runtime of the virtual machine.
/// It contains the instructions to execute, the current thread, and the ready and suspended threads.
/// The ready queue is a list of threads that are ready to run.
/// The suspended queue is a list of threads that are waiting for some event to occur.
/// The running thread is the thread that is currently executing.
/// The instructions are the bytecode instructions to execute.
pub struct Runtime {
    pub instrs: Vec<ByteCode>,
    pub running_thread: Thread,
    pub ready_queue: Vec<Thread>,
    pub suspended_queue: Vec<Thread>,
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
pub fn run(mut rt: Thread) -> Result<Thread> {
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
pub fn execute(t: &mut Thread, instr: ByteCode) -> Result<bool> {
    match instr {
        ByteCode::DONE => return Ok(true),
        ByteCode::ASSIGN(sym) => micro_code::assign(t, sym)?,
        ByteCode::LD(sym) => micro_code::ld(t, sym)?,
        ByteCode::LDC(val) => micro_code::ldc(t, val)?,
        ByteCode::LDF(addr, prms) => micro_code::ldf(t, addr, prms)?,
        ByteCode::POP => micro_code::pop(t)?,
        ByteCode::UNOP(op) => micro_code::unop(t, op)?,
        ByteCode::BINOP(op) => micro_code::binop(t, op)?,
        ByteCode::JOF(pc) => micro_code::jof(t, pc)?,
        ByteCode::GOTO(pc) => micro_code::goto(t, pc)?,
        ByteCode::RESET(ft) => micro_code::reset(t, ft)?,
        ByteCode::ENTERSCOPE(syms) => micro_code::enter_scope(t, syms)?,
        ByteCode::EXITSCOPE => micro_code::exit_scope(t)?,
        ByteCode::CALL(arity) => micro_code::call(t, arity)?,
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Ok, Result};
    use bytecode::{builtin, BinOp, ByteCode, FrameType, UnOp, Value};

    #[test]
    fn test_pc() {
        let instrs = vec![
            ByteCode::ldc(42),
            ByteCode::POP,
            ByteCode::ldc(42),
            ByteCode::POP,
            ByteCode::DONE,
        ];
        let rt = Thread::new(instrs);
        let rt = run(rt).unwrap();
        assert_eq!(rt.pc, 5);

        let rt = Thread::new(vec![
            ByteCode::ldc(false),
            ByteCode::JOF(3),
            ByteCode::POP, // This will panic since there is no value on the stack
            ByteCode::DONE,
        ]);
        let rt = run(rt).unwrap();
        assert_eq!(rt.pc, 4);

        let rt = Thread::new(vec![
            ByteCode::ldc(true),
            ByteCode::JOF(3), // jump to pop instruction
            ByteCode::DONE,
            ByteCode::POP, // This will panic since there is no value on the stack
            ByteCode::DONE,
        ]);
        let rt = run(rt).unwrap();
        assert_eq!(rt.pc, 3);

        let rt = Thread::new(vec![
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
        let rt = Thread::new(instrs);
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
        let rt = Thread::new(instrs);
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
        let rt = Thread::new(instrs);
        let rt = run(rt).unwrap();
        assert_eq!(rt.operand_stack, vec![Value::Bool(false)]);
    }

    #[test]
    fn test_assignment() {
        let instrs = vec![
            ByteCode::ldc(42),
            ByteCode::assign("x"),
            ByteCode::ldc(43),
            ByteCode::assign("y"),
            ByteCode::ldc(44),
            ByteCode::assign("x"),
            ByteCode::DONE,
        ];

        let rt = Thread::new(instrs);
        rt.env.borrow_mut().set("x", Value::Unitialized);
        rt.env.borrow_mut().set("y", Value::Unitialized);

        let rt = run(rt).unwrap();
        assert_eq!(rt.env.borrow().get(&"x".to_string()), Some(Value::Int(44)));
        assert_eq!(rt.env.borrow().get(&"y".to_string()), Some(Value::Int(43)));
    }

    #[test]
    fn test_fn_call() -> Result<()> {
        // fn simple(n) {
        //     return n;
        // }
        // simple(42)
        let instrs = vec![
            ByteCode::enterscope(vec!["simple"]),
            ByteCode::ldf(3, vec!["n"]),
            ByteCode::GOTO(5), // Jump to the end of the function
            // Body of simple
            ByteCode::ld("n"), // Load the value of n onto the stacks
            ByteCode::RESET(FrameType::CallFrame), // Return from the function
            ByteCode::assign("simple"), // Assign the function to the symbol
            ByteCode::ld("simple"), // Load the function onto the stack
            ByteCode::ldc(42), // Load the argument onto the stack
            ByteCode::CALL(1), // Call the function with 1 argument
            ByteCode::EXITSCOPE,
            ByteCode::DONE,
        ];

        let rt = Thread::new(instrs);
        let mut rt = run(rt)?;

        let result = rt.operand_stack.pop().unwrap();
        assert_eq!(result, Value::Int(42));
        assert_eq!(rt.runtime_stack.len(), 0);

        Ok(())
    }

    #[test]
    fn test_global_constants() -> Result<()> {
        let instrs = vec![ByteCode::ld(builtin::PI_SYM), ByteCode::DONE];

        let rt = Thread::new(instrs);
        let rt = run(rt)?;
        assert_eq!(rt.operand_stack, vec![Value::Float(std::f64::consts::PI)]);

        let instrs = vec![ByteCode::ld(builtin::MAX_INT_SYM), ByteCode::DONE];

        let rt = Thread::new(instrs);
        let rt = run(rt)?;

        assert_eq!(rt.operand_stack, vec![Value::Int(std::i64::MAX)]);

        Ok(())
    }

    #[test]
    fn test_global_functions() -> Result<()> {
        let instrs = vec![
            ByteCode::ld(builtin::STRING_LEN_SYM),
            ByteCode::ldc("Hello, world!"),
            ByteCode::CALL(1),
            ByteCode::DONE,
        ];

        let rt = Thread::new(instrs);
        let rt = run(rt)?;

        assert_eq!(rt.operand_stack, vec![Value::Int(13)]);

        let instrs = vec![
            ByteCode::ld(builtin::ABS_SYM),
            ByteCode::ldc(-42),
            ByteCode::CALL(1),
            ByteCode::DONE,
        ];

        let rt = Thread::new(instrs);
        let rt = run(rt)?;

        assert_eq!(rt.operand_stack, vec![Value::Int(42)]);

        Ok(())
    }
}
