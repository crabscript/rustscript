use crate::{Runtime, VmError};
use anyhow::Result;
use bytecode::{UnOp, Value};

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
        Value::Unit => {
            return Err(VmError::IllegalArgument("unit not supported".to_string()).into())
        }
        Value::Int(i) => {
            let result = match op {
                UnOp::Neg => Value::Int(-i), // Negation
                UnOp::Not => Value::Int(!i), // Bitwise Not
            };
            rt.current_thread.operand_stack.push(result);
        }
        Value::Float(f) => {
            let result = match op {
                UnOp::Neg => Value::Float(-f), // Negation
                _ => return Err(VmError::IllegalArgument("float not supported".to_string()).into()),
            };
            rt.current_thread.operand_stack.push(result);
        }
        Value::Bool(b) => {
            let result = match op {
                UnOp::Not => Value::Bool(!b), // Logical Not
                _ => return Err(VmError::IllegalArgument("bool not supported".to_string()).into()),
            };
            rt.current_thread.operand_stack.push(result);
        }
        Value::String(_) => {
            return Err(VmError::IllegalArgument("string not supported".to_string()).into())
        }
        Value::Unitialized => {
            return Err(VmError::IllegalArgument("using unitialized value".to_string()).into())
        }
        Value::Closure {
            fn_type: _,
            sym: _,
            prms: _,
            addr: _,
            env: _,
        } => return Err(VmError::IllegalArgument("closure not supported".to_string()).into()),
    }

    Ok(())
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
