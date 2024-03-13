use crate::{runtime::Runtime, VmError};
use anyhow::Result;
use bytecode::{self, BinOp, UnOp, Value};

pub fn ldc(rt: &mut Runtime, val: Value) -> Result<()> {
    rt.stack.push(val);
    Ok(())
}

pub fn pop(rt: &mut Runtime) -> Result<()> {
    rt.stack.pop().ok_or(VmError::StackUnderflow)?;
    Ok(())
}

pub fn unop(rt: &mut Runtime, op: UnOp) -> Result<()> {
    let val = rt.stack.pop().ok_or(VmError::StackUnderflow)?;

    match val {
        Value::Unit => {
            return Err(VmError::IllegalArgument("unit not supported".to_string()).into())
        }
        Value::Int(i) => {
            let result = match op {
                UnOp::Neg => Value::Int(-i), // Negation
                UnOp::Not => Value::Int(!i), // Logical negation
            };
            rt.stack.push(result);
        }
        Value::Float(f) => {
            let result = match op {
                UnOp::Neg => Value::Float(-f),
                _ => return Err(VmError::IllegalArgument("float not supported".to_string()).into()),
            };
            rt.stack.push(result);
        }
        Value::Bool(b) => {
            let result = match op {
                UnOp::Not => Value::Bool(!b),
                _ => return Err(VmError::IllegalArgument("bool not supported".to_string()).into()),
            };
            rt.stack.push(result);
        }
        Value::String(_) => {
            return Err(VmError::IllegalArgument("string not supported".to_string()).into())
        }
    }

    Ok(())
}

pub fn binop(rt: &mut Runtime, op: BinOp) -> Result<()> {
    let rhs = rt.stack.pop().ok_or(VmError::StackUnderflow)?;
    let lhs = rt.stack.pop().ok_or(VmError::StackUnderflow)?;

    match (lhs, rhs) {
        (Value::Unit, _) | (_, Value::Unit) => {
            return Err(VmError::IllegalArgument("unit not supported".to_string()).into())
        }
        (Value::Int(lhs), Value::Int(rhs)) => {
            let result = match op {
                BinOp::Add => Value::Int(lhs + rhs),
                BinOp::Sub => Value::Int(lhs - rhs),
                BinOp::Mul => Value::Int(lhs * rhs),
                BinOp::Div => Value::Int(lhs / rhs),
                BinOp::Mod => Value::Int(lhs % rhs),
                BinOp::Gt => Value::Bool(lhs > rhs),
                BinOp::Lt => Value::Bool(lhs < rhs),
                BinOp::Eq => Value::Bool(lhs == rhs),
                _ => return Err(VmError::IllegalArgument("int not supported".to_string()).into()),
            };
            rt.stack.push(result);
        }
        (Value::Float(lhs), Value::Float(rhs)) => {
            let result = match op {
                BinOp::Add => Value::Float(lhs + rhs),
                BinOp::Sub => Value::Float(lhs - rhs),
                BinOp::Mul => Value::Float(lhs * rhs),
                BinOp::Div => Value::Float(lhs / rhs),
                BinOp::Gt => Value::Bool(lhs > rhs),
                BinOp::Lt => Value::Bool(lhs < rhs),
                BinOp::Eq => Value::Bool(lhs == rhs),
                _ => return Err(VmError::IllegalArgument("float not supported".to_string()).into()),
            };
            rt.stack.push(result);
        }
        (Value::Bool(lhs), Value::Bool(rhs)) => {
            let result = match op {
                BinOp::And => Value::Bool(lhs && rhs),
                BinOp::Or => Value::Bool(lhs || rhs),
                _ => return Err(VmError::IllegalArgument("bool not supported".to_string()).into()),
            };
            rt.stack.push(result);
        }
        (Value::String(lhs), Value::String(rhs)) => {
            let result = match op {
                BinOp::Add => Value::String(lhs + &rhs),
                BinOp::Eq => Value::Bool(lhs == rhs),
                _ => {
                    return Err(VmError::IllegalArgument("string not supported".to_string()).into())
                }
            };
            rt.stack.push(result);
        }
        _ => return Err(VmError::IllegalArgument("type mismatch".to_string()).into()),
    }

    Ok(())
}

pub fn jof(rt: &mut Runtime, offset: usize) -> Result<()> {
    let cond = rt.stack.pop().ok_or(VmError::StackUnderflow)?;

    if let Value::Bool(b) = cond {
        if !b {
            rt.pc = offset;
        }

        Ok(())
    } else {
        Err(VmError::IllegalArgument("expected bool".to_string()).into())
    }
}

pub fn goto(rt: &mut Runtime, offset: usize) -> Result<()> {
    rt.pc = offset;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode::{self, Value};

    #[test]
    fn test_ldc() {
        let mut rt = Runtime::new(vec![]);
        ldc(&mut rt, Value::Unit).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Unit);

        ldc(&mut rt, Value::Int(42)).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Int(42));

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Float(42.0));

        ldc(&mut rt, Value::Bool(true)).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Bool(true));

        ldc(&mut rt, Value::String("hello world".into())).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::String("hello world".into()));
    }

    #[test]
    fn test_pop() {
        let mut rt = Runtime::new(vec![]);
        ldc(&mut rt, Value::Unit).unwrap();
        pop(&mut rt).unwrap();
        assert_eq!(rt.stack.len(), 0);

        let vals = vec![
            Value::Unit,
            Value::Int(42),
            Value::Float(42.0),
            Value::Bool(true),
            Value::String("hello world".into()),
        ];
        let val_len = vals.len();
        let mut rt = Runtime::new(vec![]);
        for val in vals {
            ldc(&mut rt, val).unwrap();
        }
        for _ in 0..val_len {
            pop(&mut rt).unwrap();
        }
        assert_eq!(rt.stack.len(), 0);

        ldc(&mut rt, Value::String("remember".into())).unwrap();
        ldc(&mut rt, Value::Unit).unwrap();
        pop(&mut rt).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::String("remember".into()));
    }

    #[test]
    fn test_unop() {
        let mut rt = Runtime::new(vec![]);
        ldc(&mut rt, Value::Int(42)).unwrap();
        unop(&mut rt, UnOp::Neg).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Int(-42));

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        unop(&mut rt, UnOp::Neg).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Float(-42.0));

        ldc(&mut rt, Value::Bool(true)).unwrap();
        unop(&mut rt, UnOp::Not).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Bool(false));

        ldc(&mut rt, Value::Bool(false)).unwrap();
        unop(&mut rt, UnOp::Not).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Bool(true));

        ldc(&mut rt, Value::Unit).unwrap();
        let result = unop(&mut rt, UnOp::Not);
        assert!(result.is_err());

        ldc(&mut rt, Value::String("hello world".into())).unwrap();
        let result = unop(&mut rt, UnOp::Not);
        assert!(result.is_err());

        ldc(&mut rt, Value::Int(42)).unwrap();
        unop(&mut rt, UnOp::Not).unwrap();
        unop(&mut rt, UnOp::Neg).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Int(43));
    }

    #[test]
    fn test_binop() {
        let mut rt = Runtime::new(vec![]);
        ldc(&mut rt, Value::Int(42)).unwrap();
        ldc(&mut rt, Value::Int(42)).unwrap();
        binop(&mut rt, BinOp::Add).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Int(84));

        ldc(&mut rt, Value::Int(1)).unwrap();
        ldc(&mut rt, Value::Int(2)).unwrap();
        binop(&mut rt, BinOp::Sub).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Int(-1));

        ldc(&mut rt, Value::Int(21)).unwrap();
        ldc(&mut rt, Value::Int(2)).unwrap();
        binop(&mut rt, BinOp::Mul).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Int(42));

        ldc(&mut rt, Value::Int(84)).unwrap();
        ldc(&mut rt, Value::Int(2)).unwrap();
        binop(&mut rt, BinOp::Div).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Int(42));

        ldc(&mut rt, Value::Int(84)).unwrap();
        ldc(&mut rt, Value::Int(2)).unwrap();
        binop(&mut rt, BinOp::Mod).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Int(0));

        ldc(&mut rt, Value::Int(84)).unwrap();
        ldc(&mut rt, Value::Int(42)).unwrap();
        binop(&mut rt, BinOp::Gt).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Bool(true));

        ldc(&mut rt, Value::Int(84)).unwrap();
        ldc(&mut rt, Value::Int(42)).unwrap();
        binop(&mut rt, BinOp::Lt).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Bool(false));

        ldc(&mut rt, Value::Int(84)).unwrap();
        ldc(&mut rt, Value::Int(42)).unwrap();
        binop(&mut rt, BinOp::Eq).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Bool(false));

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        ldc(&mut rt, Value::Int(42)).unwrap();
        let result = binop(&mut rt, BinOp::Add);
        assert!(result.is_err());

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        ldc(&mut rt, Value::Float(42.0)).unwrap();
        binop(&mut rt, BinOp::Add).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Float(84.0));

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        ldc(&mut rt, Value::Float(42.0)).unwrap();
        binop(&mut rt, BinOp::Sub).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Float(0.0));

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        ldc(&mut rt, Value::Float(42.0)).unwrap();
        binop(&mut rt, BinOp::Mul).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Float(1764.0));

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        ldc(&mut rt, Value::Float(42.0)).unwrap();
        binop(&mut rt, BinOp::Div).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Float(1.0));

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        ldc(&mut rt, Value::Float(22.0)).unwrap();
        binop(&mut rt, BinOp::Gt).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Bool(true));

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        ldc(&mut rt, Value::Float(22.0)).unwrap();
        binop(&mut rt, BinOp::Lt).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Bool(false));

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        ldc(&mut rt, Value::Float(22.0)).unwrap();
        binop(&mut rt, BinOp::Eq).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Bool(false));

        ldc(&mut rt, Value::Bool(true)).unwrap();
        ldc(&mut rt, Value::Bool(false)).unwrap();
        binop(&mut rt, BinOp::And).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Bool(false));

        ldc(&mut rt, Value::Bool(true)).unwrap();
        ldc(&mut rt, Value::Bool(false)).unwrap();
        binop(&mut rt, BinOp::Or).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Bool(true));

        ldc(&mut rt, Value::String("hello".into())).unwrap();
        ldc(&mut rt, Value::String(" world".into())).unwrap();
        binop(&mut rt, BinOp::Add).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::String("hello world".into()));

        ldc(&mut rt, Value::String("hello".into())).unwrap();
        ldc(&mut rt, Value::String(" world".into())).unwrap();
        binop(&mut rt, BinOp::Eq).unwrap();
        assert_eq!(rt.stack.pop().unwrap(), Value::Bool(false));
    }

    #[test]
    fn test_jof() {
        let mut rt = Runtime::new(vec![]);
        ldc(&mut rt, Value::Bool(false)).unwrap();
        jof(&mut rt, 123).unwrap();
        assert_eq!(rt.pc, 123);

        let mut rt = Runtime::new(vec![]);
        ldc(&mut rt, Value::Bool(true)).unwrap();
        jof(&mut rt, 42).unwrap();
        assert_eq!(rt.pc, 0);

        ldc(&mut rt, Value::Unit).unwrap();
        let result = jof(&mut rt, 42);
        assert!(result.is_err());
    }

    #[test]
    fn test_goto() {
        let mut rt = Runtime::new(vec![]);
        goto(&mut rt, 123).unwrap();
        assert_eq!(rt.pc, 123);
    }
}
