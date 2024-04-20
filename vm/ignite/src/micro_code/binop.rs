use anyhow::Result;
use bytecode::{type_of, BinOp, Value};

use crate::{Runtime, VmError};

/// Executes a binary operation on the top two values of the stack.
/// It pops the two values off the top of the stack, applies the
/// operation, and pushes the result back onto the stack.
/// Note the top of the stack is the right-hand side of the operation.
/// The second-to-top of the stack is the left-hand side of the operation.
/// The two values must be of the same type.
///
/// # Arguments
///
/// * `rt` - The runtime to execute the operation on.
///
/// * `op` - The operation to execute.
///
/// # Errors
///
/// If the stack has fewer than two values or the operation is not supported
/// for the types of the values on the stack.
#[inline]
pub fn binop(mut rt: Runtime, op: BinOp) -> Result<Runtime> {
    let rhs_val = rt
        .current_thread
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?;
    let lhs_val = rt
        .current_thread
        .operand_stack
        .pop()
        .ok_or(VmError::OperandStackUnderflow)?;

    match (lhs_val.clone(), rhs_val.clone()) {
        (Value::Unit, Value::Unit) => {
            let result = match op {
                BinOp::Eq => Value::Bool(true),
                _ => {
                    return Err(VmError::UnsupportedOperation(
                        op.into(),
                        type_of(&rhs_val).to_string(),
                    )
                    .into())
                }
            };
            rt.current_thread.operand_stack.push(result);
            Ok(rt)
        }
        (Value::Int(lhs), Value::Int(rhs)) => {
            let result = match op {
                BinOp::Add => Value::Int(lhs + rhs),  // Addition
                BinOp::Sub => Value::Int(lhs - rhs),  // Subtraction
                BinOp::Mul => Value::Int(lhs * rhs),  // Multiplication
                BinOp::Div => Value::Int(lhs / rhs),  // Division
                BinOp::Mod => Value::Int(lhs % rhs),  // Modulus
                BinOp::Gt => Value::Bool(lhs > rhs),  // Greater Than
                BinOp::Lt => Value::Bool(lhs < rhs),  // Less Than
                BinOp::Eq => Value::Bool(lhs == rhs), // Equality
                BinOp::And => {
                    return Err(VmError::UnsupportedOperation(
                        op.into(),
                        type_of(&rhs_val).to_string(),
                    )
                    .into())
                }
                BinOp::Or => {
                    return Err(VmError::UnsupportedOperation(
                        op.into(),
                        type_of(&rhs_val).to_string(),
                    )
                    .into())
                }
            };
            rt.current_thread.operand_stack.push(result);
            Ok(rt)
        }
        (Value::Float(lhs), Value::Float(rhs)) => {
            let result = match op {
                BinOp::Add => Value::Float(lhs + rhs), // Addition
                BinOp::Sub => Value::Float(lhs - rhs), // Subtraction
                BinOp::Mul => Value::Float(lhs * rhs), // Multiplication
                BinOp::Div => Value::Float(lhs / rhs), // Division
                BinOp::Gt => Value::Bool(lhs > rhs),   // Greater Than
                BinOp::Lt => Value::Bool(lhs < rhs),   // Less Than
                BinOp::Eq => Value::Bool(lhs == rhs),  // Equality
                BinOp::Or => {
                    return Err(VmError::UnsupportedOperation(
                        op.into(),
                        type_of(&rhs_val).to_string(),
                    )
                    .into())
                }
                BinOp::And => {
                    return Err(VmError::UnsupportedOperation(
                        op.into(),
                        type_of(&rhs_val).to_string(),
                    )
                    .into())
                }
                BinOp::Mod => {
                    return Err(VmError::UnsupportedOperation(
                        op.into(),
                        type_of(&rhs_val).to_string(),
                    )
                    .into())
                }
            };
            rt.current_thread.operand_stack.push(result);
            Ok(rt)
        }
        (Value::Bool(lhs), Value::Bool(rhs)) => {
            let result = match op {
                BinOp::And => Value::Bool(lhs && rhs), // Logical And
                BinOp::Or => Value::Bool(lhs || rhs),  // Logical Or
                BinOp::Eq => Value::Bool(lhs == rhs),  // Equality
                _ => {
                    return Err(VmError::UnsupportedOperation(
                        op.into(),
                        type_of(&rhs_val).to_string(),
                    )
                    .into())
                }
            };
            rt.current_thread.operand_stack.push(result);
            Ok(rt)
        }
        (Value::String(lhs), Value::String(rhs)) => {
            let result = match op {
                BinOp::Add => Value::String(lhs + &rhs),
                BinOp::Eq => Value::Bool(lhs == rhs),
                _ => {
                    return Err(VmError::UnsupportedOperation(
                        op.into(),
                        type_of(&rhs_val).to_string(),
                    )
                    .into())
                }
            };
            rt.current_thread.operand_stack.push(result);
            Ok(rt)
        }
        (Value::Semaphore(s1), Value::Semaphore(s2)) => {
            let result = match op {
                BinOp::Eq => Value::Bool(s1 == s2),
                _ => {
                    return Err(VmError::UnsupportedOperation(
                        op.into(),
                        type_of(&rhs_val).to_string(),
                    )
                    .into())
                }
            };
            rt.current_thread.operand_stack.push(result);
            Ok(rt)
        }
        (Value::Closure { .. }, Value::Closure { .. }) => {
            Err(VmError::UnsupportedOperation(op.into(), type_of(&rhs_val).to_string()).into())
        }
        _ => Err(VmError::TypeMismatch {
            expected: type_of(&lhs_val).to_string(),
            found: type_of(&rhs_val).to_string(),
        }
        .into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytecode::{BinOp, Semaphore, Value};

    use crate::micro_code::ldc;

    #[test]
    fn test_binop() {
        let mut rt = Runtime::new(vec![]);
        rt = ldc(rt, Value::Int(42)).unwrap();
        rt = ldc(rt, Value::Int(42)).unwrap();
        rt = binop(rt, BinOp::Add).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Int(84)
        );

        rt = ldc(rt, Value::Int(1)).unwrap();
        rt = ldc(rt, Value::Int(2)).unwrap();
        rt = binop(rt, BinOp::Sub).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Int(-1)
        );

        rt = ldc(rt, Value::Int(21)).unwrap();
        rt = ldc(rt, Value::Int(2)).unwrap();
        rt = binop(rt, BinOp::Mul).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Int(42)
        );

        rt = ldc(rt, Value::Int(84)).unwrap();
        rt = ldc(rt, Value::Int(2)).unwrap();
        rt = binop(rt, BinOp::Div).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Int(42)
        );

        rt = ldc(rt, Value::Int(84)).unwrap();
        rt = ldc(rt, Value::Int(2)).unwrap();
        rt = binop(rt, BinOp::Mod).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Int(0)
        );

        rt = ldc(rt, Value::Int(84)).unwrap();
        rt = ldc(rt, Value::Int(42)).unwrap();
        rt = binop(rt, BinOp::Gt).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Bool(true)
        );

        rt = ldc(rt, Value::Int(84)).unwrap();
        rt = ldc(rt, Value::Int(42)).unwrap();
        rt = binop(rt, BinOp::Lt).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Bool(false)
        );

        rt = ldc(rt, Value::Int(84)).unwrap();
        rt = ldc(rt, Value::Int(42)).unwrap();
        rt = binop(rt, BinOp::Eq).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Bool(false)
        );

        rt = ldc(rt, Value::Float(42.0)).unwrap();
        rt = ldc(rt, Value::Int(42)).unwrap();
        let result = binop(rt, BinOp::Add);
        assert!(result.is_err());

        let mut rt = Runtime::new(vec![]);
        rt = ldc(rt, Value::Float(42.0)).unwrap();
        rt = ldc(rt, Value::Float(42.0)).unwrap();
        rt = binop(rt, BinOp::Add).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Float(84.0)
        );

        rt = ldc(rt, Value::Float(42.0)).unwrap();
        rt = ldc(rt, Value::Float(42.0)).unwrap();
        rt = binop(rt, BinOp::Sub).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Float(0.0)
        );

        rt = ldc(rt, Value::Float(42.0)).unwrap();
        rt = ldc(rt, Value::Float(42.0)).unwrap();
        rt = binop(rt, BinOp::Mul).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Float(1764.0)
        );

        rt = ldc(rt, Value::Float(42.0)).unwrap();
        rt = ldc(rt, Value::Float(42.0)).unwrap();
        rt = binop(rt, BinOp::Div).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Float(1.0)
        );

        rt = ldc(rt, Value::Float(42.0)).unwrap();
        rt = ldc(rt, Value::Float(22.0)).unwrap();
        rt = binop(rt, BinOp::Gt).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Bool(true)
        );

        rt = ldc(rt, Value::Float(42.0)).unwrap();
        rt = ldc(rt, Value::Float(22.0)).unwrap();
        rt = binop(rt, BinOp::Lt).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Bool(false)
        );

        rt = ldc(rt, Value::Float(42.0)).unwrap();
        rt = ldc(rt, Value::Float(22.0)).unwrap();
        rt = binop(rt, BinOp::Eq).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Bool(false)
        );

        rt = ldc(rt, Value::Bool(true)).unwrap();
        rt = ldc(rt, Value::Bool(false)).unwrap();
        rt = binop(rt, BinOp::And).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Bool(false)
        );

        rt = ldc(rt, Value::Bool(true)).unwrap();
        rt = ldc(rt, Value::Bool(false)).unwrap();
        rt = binop(rt, BinOp::Or).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Bool(true)
        );

        rt = ldc(rt, Value::String("hello".into())).unwrap();
        rt = ldc(rt, Value::String(" world".into())).unwrap();
        rt = binop(rt, BinOp::Add).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::String("hello world".into())
        );

        rt = ldc(rt, Value::String("hello".into())).unwrap();
        rt = ldc(rt, Value::String(" world".into())).unwrap();
        rt = binop(rt, BinOp::Eq).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Bool(false)
        );

        let sem: Value = Semaphore::new(1).into();
        rt = ldc(rt, sem.clone()).unwrap();
        rt = ldc(rt, sem).unwrap();
        rt = binop(rt, BinOp::Eq).unwrap();
        assert_eq!(
            rt.current_thread.operand_stack.pop().unwrap(),
            Value::Bool(true)
        );
    }
}
