use crate::{Runtime, VmError};
use anyhow::Result;
use bytecode::{BinOp, Value};

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
pub fn binop(rt: &mut Runtime, op: BinOp) -> Result<()> {
    let rhs = rt.operand_stack.pop().ok_or(VmError::StackUnderflow)?;
    let lhs = rt.operand_stack.pop().ok_or(VmError::StackUnderflow)?;

    match (lhs, rhs) {
        (Value::Unit, Value::Unit) => {
            let result = match op {
                BinOp::Eq => Value::Bool(true),
                _ => return Err(VmError::IllegalArgument("unit not supported".to_string()).into()),
            };
            rt.operand_stack.push(result);
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
                    return Err(
                        VmError::IllegalArgument("integer not supported".to_string()).into(),
                    )
                }
                BinOp::Or => {
                    return Err(
                        VmError::IllegalArgument("integer not supported".to_string()).into(),
                    )
                }
            };
            rt.operand_stack.push(result);
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
                    return Err(VmError::IllegalArgument("float not supported".to_string()).into())
                }
                BinOp::And => {
                    return Err(VmError::IllegalArgument("float not supported".to_string()).into())
                }
                BinOp::Mod => {
                    return Err(VmError::IllegalArgument("float not supported".to_string()).into())
                }
            };
            rt.operand_stack.push(result);
        }
        (Value::Bool(lhs), Value::Bool(rhs)) => {
            let result = match op {
                BinOp::And => Value::Bool(lhs && rhs), // Logical And
                BinOp::Or => Value::Bool(lhs || rhs),  // Logical Or
                BinOp::Eq => Value::Bool(lhs == rhs),  // Equality
                _ => return Err(VmError::IllegalArgument("bool not supported".to_string()).into()),
            };
            rt.operand_stack.push(result);
        }
        (Value::String(lhs), Value::String(rhs)) => {
            let result = match op {
                BinOp::Add => Value::String(lhs + &rhs),
                BinOp::Eq => Value::Bool(lhs == rhs),
                _ => {
                    return Err(VmError::IllegalArgument("string not supported".to_string()).into())
                }
            };
            rt.operand_stack.push(result);
        }
        _ => return Err(VmError::IllegalArgument("type mismatch".to_string()).into()),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::micro_code::ldc;
    use crate::Runtime;
    use bytecode::{BinOp, Value};

    #[test]
    fn test_binop() {
        let mut rt = Runtime::new(vec![]);
        ldc(&mut rt, Value::Int(42)).unwrap();
        ldc(&mut rt, Value::Int(42)).unwrap();
        binop(&mut rt, BinOp::Add).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Int(84));

        ldc(&mut rt, Value::Int(1)).unwrap();
        ldc(&mut rt, Value::Int(2)).unwrap();
        binop(&mut rt, BinOp::Sub).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Int(-1));

        ldc(&mut rt, Value::Int(21)).unwrap();
        ldc(&mut rt, Value::Int(2)).unwrap();
        binop(&mut rt, BinOp::Mul).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Int(42));

        ldc(&mut rt, Value::Int(84)).unwrap();
        ldc(&mut rt, Value::Int(2)).unwrap();
        binop(&mut rt, BinOp::Div).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Int(42));

        ldc(&mut rt, Value::Int(84)).unwrap();
        ldc(&mut rt, Value::Int(2)).unwrap();
        binop(&mut rt, BinOp::Mod).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Int(0));

        ldc(&mut rt, Value::Int(84)).unwrap();
        ldc(&mut rt, Value::Int(42)).unwrap();
        binop(&mut rt, BinOp::Gt).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Bool(true));

        ldc(&mut rt, Value::Int(84)).unwrap();
        ldc(&mut rt, Value::Int(42)).unwrap();
        binop(&mut rt, BinOp::Lt).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Bool(false));

        ldc(&mut rt, Value::Int(84)).unwrap();
        ldc(&mut rt, Value::Int(42)).unwrap();
        binop(&mut rt, BinOp::Eq).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Bool(false));

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        ldc(&mut rt, Value::Int(42)).unwrap();
        let result = binop(&mut rt, BinOp::Add);
        assert!(result.is_err());

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        ldc(&mut rt, Value::Float(42.0)).unwrap();
        binop(&mut rt, BinOp::Add).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Float(84.0));

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        ldc(&mut rt, Value::Float(42.0)).unwrap();
        binop(&mut rt, BinOp::Sub).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Float(0.0));

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        ldc(&mut rt, Value::Float(42.0)).unwrap();
        binop(&mut rt, BinOp::Mul).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Float(1764.0));

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        ldc(&mut rt, Value::Float(42.0)).unwrap();
        binop(&mut rt, BinOp::Div).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Float(1.0));

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        ldc(&mut rt, Value::Float(22.0)).unwrap();
        binop(&mut rt, BinOp::Gt).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Bool(true));

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        ldc(&mut rt, Value::Float(22.0)).unwrap();
        binop(&mut rt, BinOp::Lt).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Bool(false));

        ldc(&mut rt, Value::Float(42.0)).unwrap();
        ldc(&mut rt, Value::Float(22.0)).unwrap();
        binop(&mut rt, BinOp::Eq).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Bool(false));

        ldc(&mut rt, Value::Bool(true)).unwrap();
        ldc(&mut rt, Value::Bool(false)).unwrap();
        binop(&mut rt, BinOp::And).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Bool(false));

        ldc(&mut rt, Value::Bool(true)).unwrap();
        ldc(&mut rt, Value::Bool(false)).unwrap();
        binop(&mut rt, BinOp::Or).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Bool(true));

        ldc(&mut rt, Value::String("hello".into())).unwrap();
        ldc(&mut rt, Value::String(" world".into())).unwrap();
        binop(&mut rt, BinOp::Add).unwrap();
        assert_eq!(
            rt.operand_stack.pop().unwrap(),
            Value::String("hello world".into())
        );

        ldc(&mut rt, Value::String("hello".into())).unwrap();
        ldc(&mut rt, Value::String(" world".into())).unwrap();
        binop(&mut rt, BinOp::Eq).unwrap();
        assert_eq!(rt.operand_stack.pop().unwrap(), Value::Bool(false));
    }
}
