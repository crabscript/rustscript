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
pub fn unop(mut rt: Runtime, op: UnOp) -> Result<Runtime> {
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
            Ok(rt)
        }
        Value::Float(f) => {
            if let UnOp::Neg = op {
                let result = Value::Float(-f); // Negation
                rt.current_thread.operand_stack.push(result);
                Ok(rt)
            } else {
                Err(VmError::UnsupportedOperation(op.into(), type_of(&val).into()).into())
            }
        }
        Value::Bool(b) => {
            if let UnOp::Not = op {
                let result = Value::Bool(!b); // Logical Not
                rt.current_thread.operand_stack.push(result);
                Ok(rt)
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
        rt = ldc(rt, Value::Int(42)).unwrap();
        rt = unop(rt, UnOp::Neg).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Int(-42)
        );

        rt = ldc(rt, Value::Float(42.0)).unwrap();
        rt = unop(rt, UnOp::Neg).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Float(-42.0)
        );

        rt = ldc(rt, Value::Bool(true)).unwrap();
        rt = unop(rt, UnOp::Not).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Bool(false)
        );

        rt = ldc(rt, Value::Bool(false)).unwrap();
        rt = unop(rt, UnOp::Not).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Bool(true)
        );

        rt = ldc(rt, Value::Unit).unwrap();
        let result = unop(rt, UnOp::Not);
        assert!(result.is_err());

        let mut rt = Runtime::new(vec![]);
        rt = ldc(rt, Value::String("hello world".into())).unwrap();
        let result = unop(rt, UnOp::Not);
        assert!(result.is_err());

        let mut rt = Runtime::new(vec![]);
        rt = ldc(rt, Value::Int(42)).unwrap();
        rt = unop(rt, UnOp::Not).unwrap();
        rt = unop(rt, UnOp::Neg).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Int(43)
        );
    }
}
