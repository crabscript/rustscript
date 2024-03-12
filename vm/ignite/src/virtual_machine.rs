use anyhow::Result;
use bytecode::{self, ByteCode, Value};

use crate::VmError;

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

    fn execute(&mut self, instr: ByteCode) -> Result<()> {
        match instr {
            ByteCode::LDC(val) => self.stack.push(val),
            ByteCode::UNOP(op) => {
                let val = self.stack.pop().ok_or(VmError::StackUnderflow)?;

                match val {
                    Value::Int(i) => {
                        let result = match op {
                            bytecode::UnOp::Neg => Value::Int(-i), // Negation
                            bytecode::UnOp::Not => Value::Int(!i), // Logical negation
                        };
                        self.stack.push(result);
                    }
                    Value::Float(f) => {
                        let result = match op {
                            bytecode::UnOp::Neg => Value::Float(-f),
                            _ => return Err(VmError::Unimplemented.into()),
                        };
                        self.stack.push(result);
                    }
                    Value::Bool(b) => {
                        let result = match op {
                            bytecode::UnOp::Not => Value::Bool(!b),
                            _ => return Err(VmError::Unimplemented.into()),
                        };
                        self.stack.push(result);
                    }
                }
            }
            _ => unimplemented!(),
        }

        Ok(())
    }
}

pub fn run(mut rt: Runtime) -> Result<Value> {
    while rt.pc < rt.instrs.len() {
        let instr = rt.instrs[rt.pc].clone();
        rt.pc += 1;
        rt.execute(instr)?
    }

    Ok(rt.stack.pop().ok_or(VmError::StackUnderflow)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unop_neg() {
        let instrs = vec![
            ByteCode::LDC(Value::Int(42)),
            ByteCode::UNOP(bytecode::UnOp::Neg),
        ];
        let rt = Runtime::new(instrs);
        let result = run(rt).unwrap();

        assert_eq!(result, Value::Int(-42));
    }

    #[test]
    fn test_unop_not() {
        let instrs = vec![
            ByteCode::LDC(Value::Int(42)),
            ByteCode::UNOP(bytecode::UnOp::Not),
        ];
        let rt = Runtime::new(instrs);
        let result = run(rt).unwrap();

        assert_eq!(result, Value::Int(-43));
    }

    #[test]
    fn test_unop_neg_float() {
        let instrs = vec![
            ByteCode::LDC(Value::Float(42.0)),
            ByteCode::UNOP(bytecode::UnOp::Neg),
        ];
        let rt = Runtime::new(instrs);
        let result = run(rt).unwrap();

        assert_eq!(result, Value::Float(-42.0));
    }

    #[test]
    fn test_unop_not_bool() {
        let instrs = vec![
            ByteCode::LDC(Value::Bool(true)),
            ByteCode::UNOP(bytecode::UnOp::Not),
        ];
        let rt = Runtime::new(instrs);
        let result = run(rt).unwrap();

        assert_eq!(result, Value::Bool(false));
    }
}
