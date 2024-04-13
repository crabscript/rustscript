use crate::{Runtime, VmError};
use anyhow::{Ok, Result};
use bytecode::{type_of, UnOp, Value};

/// Executes a unary operation on the top of the stack.
/// It pops the value off the top of the stack, applies the
/// operation, and pushes the result back onto the stack.
///
/// # Arguments
///
/// * `rt` - The runtime to execute the operation on.
///
/// * `op` - The operation to execute.
///
/// # Errors
///
/// If the stack is empty or the operation is not supported for
/// the type of the value on the stack.
pub fn unop(rt: &mut Runtime, op: UnOp) -> Result<()> {
    let val = rt
        .current_thread
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?;

    match val {
        Value::Unit => Err(VmError::UnsupportedOperation(op.into(), type_of(&val).into()).into()),
        Value::Int(i) => {
            let result = match op {
                UnOp::Neg => Value::Int(-i), // Negation
                UnOp::Not => Value::Int(!i), // Bitwise Not
            };
            rt.current_thread.operand_stack.push(result);
            Ok(())
        }
        Value::Float(f) => {
            if let UnOp::Neg = op {
                let result = Value::Float(-f); // Negation
                rt.current_thread.operand_stack.push(result);
                Ok(())
            } else {
                Err(VmError::UnsupportedOperation(op.into(), type_of(&val).into()).into())
            }
        }
        Value::Bool(b) => {
            if let UnOp::Not = op {
                let result = Value::Bool(!b); // Logical Not
                rt.current_thread.operand_stack.push(result);
                Ok(())
            } else {
                Err(VmError::UnsupportedOperation(op.into(), type_of(&val).into()).into())
            }
        }
        Value::String(_) => {
            Err(VmError::UnsupportedOperation(op.into(), type_of(&val).into()).into())
        }
        Value::Unitialized => {
            Err(VmError::UnsupportedOperation(op.into(), type_of(&val).into()).into())
        }
        Value::Semaphore(_) => {
            Err(VmError::UnsupportedOperation(op.into(), type_of(&val).into()).into())
        }
        Value::Closure { .. } => {
            Err(VmError::UnsupportedOperation(op.into(), type_of(&val).into()).into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::micro_code::ldc;
    use bytecode::{UnOp, Value};

    #[test]
    fn test_unop() {
        let mut rt = Runtime::new(vec![]);
        ldc(&mut rt, Value::Int(42)).unwrap();
        unop(&mut rt, UnOp::Neg).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Int(-42)
        );

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        unop(&mut rt, UnOp::Neg).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Float(-42.0)
        );

        ldc(&mut rt, Value::Bool(true)).unwrap();
        unop(&mut rt, UnOp::Not).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Bool(false)
        );

        ldc(&mut rt, Value::Bool(false)).unwrap();
        unop(&mut rt, UnOp::Not).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Bool(true)
        );

        ldc(&mut rt, Value::Unit).unwrap();
        let result = unop(&mut rt, UnOp::Not);
        assert!(result.is_err());

        ldc(&mut rt, Value::String("hello world".into())).unwrap();
        let result = unop(&mut rt, UnOp::Not);
        assert!(result.is_err());

        ldc(&mut rt, Value::Int(42)).unwrap();
        unop(&mut rt, UnOp::Not).unwrap();
        unop(&mut rt, UnOp::Neg).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Int(43)
        );
    }
}
