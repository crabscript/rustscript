use anyhow::Result;
use bytecode::{self, ByteCode, Value};

use crate::micro_code;

/// The runtime for each thread of execution.
#[derive(Debug, Default)]
pub struct Runtime {
    pub stack: Vec<Value>,
    pub instrs: Vec<ByteCode>,
    pub pc: usize,
}

impl Runtime {
    pub fn new(instrs: Vec<ByteCode>) -> Self {
        Runtime {
            stack: Vec::new(),
            instrs,
            ..Default::default()
        }
    }
}

pub fn run(mut rt: Runtime) -> Result<Runtime> {
    while rt.pc < rt.instrs.len() {
        let instr = rt.instrs[rt.pc].clone();
        rt.pc += 1;
        execute(&mut rt, instr)?;
    }

    Ok(rt)
}

pub fn execute(rt: &mut Runtime, instr: ByteCode) -> Result<()> {
    match instr {
        ByteCode::LDC(val) => micro_code::ldc(rt, val)?,
        ByteCode::POP => micro_code::pop(rt)?,
        ByteCode::UNOP(op) => micro_code::unop(rt, op)?,
        ByteCode::BINOP(op) => micro_code::binop(rt, op)?,
        ByteCode::JOF(pc) => micro_code::jof(rt, pc)?,
        ByteCode::GOTO(pc) => micro_code::goto(rt, pc)?,
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pc() {
        let instrs = vec![
            ByteCode::ldc(42),
            ByteCode::POP,
            ByteCode::ldc(42),
            ByteCode::POP,
        ];
        let rt = Runtime::new(instrs);
        let rt = run(rt).unwrap();
        assert_eq!(rt.pc, 4);

        let rt = Runtime::new(vec![
            ByteCode::ldc(42),
            ByteCode::ldc(false),
            ByteCode::JOF(0),
        ]);
        let rt = run(rt).unwrap();
        assert_eq!(rt.pc, 0);

        let rt = Runtime::new(vec![
            ByteCode::ldc(42),
            ByteCode::ldc(true),
            ByteCode::JOF(0),
        ]);
        let rt = run(rt).unwrap();
        assert_eq!(rt.pc, 3);

        let rt = Runtime::new(vec![ByteCode::ldc(42), ByteCode::POP, ByteCode::GOTO(0)]);
        let rt = run(rt).unwrap();
        assert_eq!(rt.pc, 0);
    }
}
