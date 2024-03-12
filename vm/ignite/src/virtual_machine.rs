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
}

pub fn run(mut rt: Runtime) -> Result<Value> {
    while rt.pc < rt.instrs.len() {
        let instr = rt.instrs[rt.pc].clone();
        rt.pc += 1;
        execute(&mut rt, instr)?;
    }

    Ok(rt.stack.pop().ok_or(VmError::StackUnderflow)?)
}

pub fn execute(rt: &mut Runtime, instr: ByteCode) -> Result<()> {
    match instr {
        ByteCode::LDC(val) => rt.stack.push(val),
        ByteCode::UNOP(op) => {
            let val = rt.stack.pop().ok_or(VmError::StackUnderflow)?;

            match val {
                Value::Int(i) => {
                    let result = match op {
                        bytecode::UnOp::Neg => Value::Int(-i), // Negation
                        bytecode::UnOp::Not => Value::Int(!i), // Logical negation
                    };
                    rt.stack.push(result);
                }
                Value::Float(f) => {
                    let result = match op {
                        bytecode::UnOp::Neg => Value::Float(-f),
                        _ => {
                            return Err(
                                VmError::IllegalArgument("float not supported".to_string()).into()
                            )
                        }
                    };
                    rt.stack.push(result);
                }
                Value::Bool(b) => {
                    let result = match op {
                        bytecode::UnOp::Not => Value::Bool(!b),
                        _ => {
                            return Err(
                                VmError::IllegalArgument("bool not supported".to_string()).into()
                            )
                        }
                    };
                    rt.stack.push(result);
                }
            }
        }
        ByteCode::BINOP(op) => {
            let rhs = rt.stack.pop().ok_or(VmError::StackUnderflow)?;
            let lhs = rt.stack.pop().ok_or(VmError::StackUnderflow)?;

            match (lhs, rhs) {
                (Value::Int(lhs), Value::Int(rhs)) => {
                    let result = match op {
                        bytecode::BinOp::Add => Value::Int(lhs + rhs),
                        bytecode::BinOp::Sub => Value::Int(lhs - rhs),
                        bytecode::BinOp::Mul => Value::Int(lhs * rhs),
                        bytecode::BinOp::Div => Value::Int(lhs / rhs),
                        bytecode::BinOp::Mod => Value::Int(lhs % rhs),
                        bytecode::BinOp::Gt => Value::Bool(lhs > rhs),
                        bytecode::BinOp::Lt => Value::Bool(lhs < rhs),
                        bytecode::BinOp::Eq => Value::Bool(lhs == rhs),
                        _ => {
                            return Err(
                                VmError::IllegalArgument("int not supported".to_string()).into()
                            )
                        }
                    };
                    rt.stack.push(result);
                }
                (Value::Float(lhs), Value::Float(rhs)) => {
                    let result = match op {
                        bytecode::BinOp::Add => Value::Float(lhs + rhs),
                        bytecode::BinOp::Sub => Value::Float(lhs - rhs),
                        bytecode::BinOp::Mul => Value::Float(lhs * rhs),
                        bytecode::BinOp::Div => Value::Float(lhs / rhs),
                        bytecode::BinOp::Mod => Value::Float(lhs % rhs),
                        bytecode::BinOp::Gt => Value::Bool(lhs > rhs),
                        bytecode::BinOp::Lt => Value::Bool(lhs < rhs),
                        bytecode::BinOp::Eq => Value::Bool(lhs == rhs),
                        _ => {
                            return Err(
                                VmError::IllegalArgument("float not supported".to_string()).into()
                            )
                        }
                    };
                    rt.stack.push(result);
                }
                (Value::Bool(lhs), Value::Bool(rhs)) => {
                    let result = match op {
                        bytecode::BinOp::And => Value::Bool(lhs && rhs),
                        bytecode::BinOp::Or => Value::Bool(lhs || rhs),
                        _ => {
                            return Err(
                                VmError::IllegalArgument("bool not supported".to_string()).into()
                            )
                        }
                    };
                    rt.stack.push(result);
                }
                _ => return Err(VmError::IllegalArgument("type mismatch".to_string()).into()),
            }
        } // _ => unimplemented!(),
    }

    Ok(())
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

    #[test]
    fn test_longer_unop_chain() {
        let instrs = vec![
            ByteCode::LDC(Value::Int(42)),
            ByteCode::UNOP(bytecode::UnOp::Neg),
            ByteCode::UNOP(bytecode::UnOp::Neg),
            ByteCode::UNOP(bytecode::UnOp::Neg),
        ];
        let rt = Runtime::new(instrs);
        let result = run(rt).unwrap();

        assert_eq!(result, Value::Int(-42));
    }

    #[test]
    fn test_binop_add() {
        let instrs = vec![
            ByteCode::LDC(Value::Int(40)),
            ByteCode::LDC(Value::Int(2)),
            ByteCode::BINOP(bytecode::BinOp::Add),
        ];
        let rt = Runtime::new(instrs);
        let result = run(rt).unwrap();

        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_binop_sub() {
        let instrs = vec![
            ByteCode::LDC(Value::Int(44)),
            ByteCode::LDC(Value::Int(2)),
            ByteCode::BINOP(bytecode::BinOp::Sub),
        ];
        let rt = Runtime::new(instrs);
        let result = run(rt).unwrap();

        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_binop_mul() {
        let instrs = vec![
            ByteCode::LDC(Value::Int(21)),
            ByteCode::LDC(Value::Int(2)),
            ByteCode::BINOP(bytecode::BinOp::Mul),
        ];
        let rt = Runtime::new(instrs);
        let result = run(rt).unwrap();

        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_binop_div() {
        let instrs = vec![
            ByteCode::LDC(Value::Int(84)),
            ByteCode::LDC(Value::Int(2)),
            ByteCode::BINOP(bytecode::BinOp::Div),
        ];
        let rt = Runtime::new(instrs);
        let result = run(rt).unwrap();

        assert_eq!(result, Value::Int(42));
    }

    #[test]
    fn test_binop_mod() {
        let instrs = vec![
            ByteCode::LDC(Value::Int(84)),
            ByteCode::LDC(Value::Int(42)),
            ByteCode::BINOP(bytecode::BinOp::Mod),
        ];
        let rt = Runtime::new(instrs);
        let result = run(rt).unwrap();

        assert_eq!(result, Value::Int(0));
    }

    #[test]
    fn test_binop_gt() {
        let instrs = vec![
            ByteCode::LDC(Value::Int(84)),
            ByteCode::LDC(Value::Int(42)),
            ByteCode::BINOP(bytecode::BinOp::Gt),
        ];
        let rt = Runtime::new(instrs);
        let result = run(rt).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binop_lt() {
        let instrs = vec![
            ByteCode::LDC(Value::Int(42)),
            ByteCode::LDC(Value::Int(84)),
            ByteCode::BINOP(bytecode::BinOp::Lt),
        ];
        let rt = Runtime::new(instrs);
        let result = run(rt).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binop_eq() {
        let instrs = vec![
            ByteCode::LDC(Value::Int(42)),
            ByteCode::LDC(Value::Int(42)),
            ByteCode::BINOP(bytecode::BinOp::Eq),
        ];
        let rt = Runtime::new(instrs);
        let result = run(rt).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binop_and() {
        let instrs = vec![
            ByteCode::LDC(Value::Bool(true)),
            ByteCode::LDC(Value::Bool(true)),
            ByteCode::BINOP(bytecode::BinOp::And),
        ];
        let rt = Runtime::new(instrs);
        let result = run(rt).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_binop_or() {
        let instrs = vec![
            ByteCode::LDC(Value::Bool(true)),
            ByteCode::LDC(Value::Bool(false)),
            ByteCode::BINOP(bytecode::BinOp::Or),
        ];
        let rt = Runtime::new(instrs);
        let result = run(rt).unwrap();

        assert_eq!(result, Value::Bool(true));
    }

    #[test]
    fn test_longer_binop_chain() {
        let instrs = vec![
            ByteCode::LDC(Value::Int(40)),
            ByteCode::LDC(Value::Int(2)),
            ByteCode::BINOP(bytecode::BinOp::Add),
            ByteCode::LDC(Value::Int(2)),
            ByteCode::BINOP(bytecode::BinOp::Mul),
        ];
        let rt = Runtime::new(instrs);
        let result = run(rt).unwrap();

        assert_eq!(result, Value::Int(84));
    }

    #[test]
    fn test_unop_binop_chain() {
        let instrs = vec![
            ByteCode::LDC(Value::Int(40)),
            ByteCode::UNOP(bytecode::UnOp::Neg),
            ByteCode::LDC(Value::Int(2)),
            ByteCode::BINOP(bytecode::BinOp::Add),
        ];
        let rt = Runtime::new(instrs);
        let result = run(rt).unwrap();

        assert_eq!(result, Value::Int(-38));
    }
}
