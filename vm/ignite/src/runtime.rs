use crate::{frame::Frame, micro_code, VmError};
use anyhow::Result;
use bytecode::{self, ByteCode, Value};

/// The runtime for each thread of execution.
#[derive(Debug, Default)]
pub struct Runtime {
    pub frame: Frame,
    pub operand_stack: Vec<Value>,
    pub instrs: Vec<ByteCode>,
    pub pc: usize,
}

impl Runtime {
    pub fn new(instrs: Vec<ByteCode>) -> Self {
        Runtime {
            frame: Frame::new(None),
            operand_stack: Vec::new(),
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
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
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
        assert_eq!(rt.frame.get(&"x".to_string()), Some(&Value::Int(44)));
        assert_eq!(rt.frame.get(&"y".to_string()), Some(&Value::Int(43)));
    }
}
